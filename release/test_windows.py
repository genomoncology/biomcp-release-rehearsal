"""Release contract checks use PE fixtures without a Windows toolchain."""
import configparser
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

from release.artifacts import wheel
from release.test_licenses import write_payloads


def pe(imports, delayed=False):
    data = bytearray(2048)
    data[:2] = b'MZ'
    struct.pack_into('<I', data, 0x3c, 128)
    data[128:132] = b'PE\0\0'
    struct.pack_into('<HH', data, 132, 0x8664, 1)
    struct.pack_into('<H', data, 148, 240)
    struct.pack_into('<H', data, 152, 0x20b)
    struct.pack_into('<I', data, 152 + 108, 16)
    size = 32 if delayed else 20
    struct.pack_into('<II', data, 152 + 112 + 8 * (13 if delayed else 1), 0x1000, size * (len(imports) + 1))
    struct.pack_into('<IIII', data, 392 + 8, 1536, 0x1000, 1536, 512)
    offset = 1024
    for index, name in enumerate(imports):
        descriptor = 512 + index * size
        struct.pack_into('<I', data, descriptor + (4 if delayed else 12), 0x1000 + offset - 512)
        if delayed:
            struct.pack_into('<I', data, descriptor, 1)
        encoded = name.encode('ascii') + b'\0'
        data[offset:offset + len(encoded)] = encoded
        offset += len(encoded)
    return data


class WindowsRuntime(unittest.TestCase):
    def test_installed_gate_refuses_redistributable_before_execution(self):
        for name in ('VCRUNTIME140.dll', 'vcruntime140_1.DLL', 'MSVCP140.dll', 'CONCRT140.dll', 'VCOMP140.DLL', 'MFC140U.dll', 'ATl140.dll',
                     r'.\VCRUNTIME140.dll', 'runtime/VCRUNTIME140.dll', r'C:\runtime\VCRUNTIME140.dll', 'VCRUNTIME140'):
            for delayed in (False, True):
                with self.subTest(name=name, delayed=delayed), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    binary = root / 'biomcp.exe'
                    binary.write_bytes(pe(['KERNEL32.dll', name], delayed))
                    record = {'distribution': 'biomcp-cli', 'version': '1.0.0', 'platform': 'win_amd64',
                              'profile': 'release', 'cargo_profile': {'debug_assertions': False, 'opt_level': '3'},
                              'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'revision': 'a' * 40}
                    write_payloads(root)
                    wheel(root, binary, record)
                    result = subprocess.run([sys.executable, 'release/gate.py', '--prepared', '--artifacts', str(root),
                                             '--scratch', str(root / 'scratch')], capture_output=True, text=True, timeout=20)
                    self.assertNotEqual(result.returncode, 0)
                    expected = ('Windows binary requires Visual C++ redistributable: ' + name
                                if name.lower().endswith('.dll') and not any(char in name for char in '/\\:')
                                else 'invalid Windows PE: unsupported DLL name')
                    self.assertIn(expected, result.stderr)

    def test_windows_system_imports_pass_and_malformed_images_fail_closed(self):
        from release.windows import check_runtime
        check_runtime(pe(['KERNEL32.dll', 'ucrtbase.dll', 'api-ms-win-crt-runtime-l1-1-0.dll']), 'win_amd64')
        check_runtime(b'not a PE image', 'linux_x86_64')
        for data in (b'', pe(['KERNEL32.dll'])[:600]):
            with self.subTest(length=len(data)), self.assertRaisesRegex(ValueError, 'invalid Windows PE'):
                check_runtime(data, 'win_amd64')

    def test_static_runtime_configuration_is_target_specific(self):
        config = configparser.ConfigParser()
        config.read('.cargo/config.toml')
        target = 'target.x86_64-pc-windows-msvc'
        self.assertEqual(config.sections(), [target, 'net'])
        self.assertEqual(list(config['net']), ['git-fetch-with-cli'])
        self.assertTrue(config.getboolean('net', 'git-fetch-with-cli'))
        self.assertEqual(list(config[target]), ['rustflags'])
        self.assertEqual(json.loads(config[target]['rustflags']), ['-C', 'target-feature=+crt-static'])


if __name__ == '__main__':
    unittest.main()
