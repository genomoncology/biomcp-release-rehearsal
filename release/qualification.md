# Qualify release channels

Run the checked, reviewed tooling commit separately from the product commit under test. For the final 1.0 rehearsal the artifact source is `ed9c36a963ef79317ac2568b495aea443e6d079e`. Do not substitute the runner's HEAD. These commands publish nothing. Run them as tracked `pm job` commands from the approved tooling branch; retain each job's log as that channel's evidence. A missing executable, missing archive or failed command is a failed or not-run channel, never a pass.

## Pinned public 0.9 installer

On a native Unix host, point `--artifacts` at that platform's existing producer output containing the wheel, `release.json`, native archive and checksum:

```sh
python3 release/published.py legacy-installer --version 1.0.0 --revision ed9c36a963ef79317ac2568b495aea443e6d079e --artifacts /path/to/native-artifacts
```

The runner fetches `install.sh` from public 0.9.3 commit `5641de7d7b79b53872f8599dbb4dba22e611dcaf`, requires SHA-256 `8d4592de9b4ac55958b22282620b234fcdeac3f248211265a3da144adf47a34a`, and runs those exact bytes. `--installer-script PATH` can supply a cached copy with the same required hash. That installer has no configurable download base, so a curl adapter redirects only its exact requested release archive and checksum to the existing ephemeral loopback server in `release/local_server.py`. Other URLs are refused. The installed executable must match the inspected candidate wheel's binary hash and return the requested version. The log records the installer pin, archive hash and artifact source commit.

## Claude plugin

Unpack the staged candidate `biomcp-plugin.zip` into a folder and provide an executable extracted from the matching native candidate:

```sh
python3 release/published.py plugin --version 1.0.0 --revision ed9c36a963ef79317ac2568b495aea443e6d079e --artifacts /path/to/native-artifacts --plugin-source /path/to/unpacked-candidate-plugin --binary /path/to/installed/biomcp --claude /path/to/claude
```

The real Claude CLI adds the local marketplace and installs `biomcp@biomcp` into a temporary home. The runner reads Claude's installed-plugin registry, checks the installed manifest and MCP launch declaration, then starts that declaration's `biomcp serve` using the matching installed executable. It initializes MCP, lists tools and calls `biomcp` with `help`, requiring actual help commands in the returned result. This needs no model invocation or source API call. The temporary home and plugin are removed on success or failure. The log records the command result and candidate executable identity.

## Windows wrapper

On the existing Windows VM's host, use the current owning qualification ticket (`0293` or `0621`) and the existing Windows candidate artifact folder:

```sh
sh scripts/check_windows_release.sh 0293 ed9c36a963ef79317ac2568b495aea443e6d079e /path/to/windows-artifacts
```

The wrapper requires the PM job's ticket label to match its first argument. It takes the existing VM lock under the actual job ID, refuses a running VM, copies the reviewed tooling and candidate artifacts into a job-specific folder, and runs the ordinary installed release gate. Before the gate, the inspected wheel's producer identity must equal its sidecar and the requested product commit. The gate exercises the installed Windows distributions; no old HGNC or config ticket tests are substituted. Cleanup removes only the job's folder, stops the VM and releases its own lock. Keep that PM job log as the wrapper evidence.

## Five native platforms

Use the already approved public hosted rehearsal of the final product commit, under a non-`v` tag with `rehearsal=true` and publication off. Its five native jobs cover Linux x86_64, Linux arm64, macOS arm64, Intel macOS and Windows. Retain the hosted run URL, native job results and attested artifact identities separately from the local channel logs. The local wrappers and channel checks do not establish this five-platform result. This tooling change does not dispatch the rehearsal, append public history, tag a release or publish a channel.

## Development checks

`sh scripts/check_release_channels.sh` runs focused runner tests, including a real Claude install against a tiny test MCP server and the pinned installer against a tiny local candidate. These check the tooling and do not qualify the release executable. `sh scripts/check_release_channels.sh routine` runs the repository's canonical routine gate.
