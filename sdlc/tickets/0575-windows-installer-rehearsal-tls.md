# Windows installer download proof trusts its fixture CA

Status: COMPLETE.
Landed: d0e6c606c102cf8193984ca93e460956d69b81e6
Milestone: 5

## Outcome

On Windows, the installed gate's download-install proof downloads the release archive from its local HTTPS fixture with normal certificate verification on, trusting only the fixture's named CA, and `release/install.sh --version` installs a working binary.

- Why: private rehearsal run 41388677743 at dac98145 passed four installed-artifact rounds on Windows (523 of 523 each, both Windows configuration path cases included), then failed at the download proof: curl exit 60, Schannel `SEC_E_UNTRUSTED_ROOT` (0x80090325), "could not download release archive" (Yellow `biomcp-redux-evidence/0294/private-rehearsal-41388677743/windows-job.log` lines 3550-3566). `release/gate.py:189-191` passes the fixture CA as `CURL_CA_BUNDLE` to Git Bash's `install.sh`. One hypothesis, not yet shown: Schannel curl uses the Windows certificate store and ignores the bundle, or the CA path's form is wrong for it.
- Starts from: `release/gate.py` (`https_server`, `installer_environment`, the download step), `release/install.sh`'s curl call, and the raw log.
- Rules: bounded diagnosis first: name which curl ran, its TLS backend, and whether it read the CA file. Fix by giving that curl the fixture CA in a form it honors, such as `--cacert` with the right path form, or by choosing a curl whose backend honors the bundle. Never disable verification, never add the fixture CA to the machine's trust store, and change no account or runner setting. Real users' downloads from the public release keep using the system's normal trust.
- Proof: a fixture test that fails when the installer's curl would not receive the fixture CA, and the next private rehearsal's Windows job passes the download-install step.
- Changes: the gate's download step and, if needed, how `install.sh` passes a CA bundle to curl.
- Defers: other rehearsal failures, each to its own ticket.
