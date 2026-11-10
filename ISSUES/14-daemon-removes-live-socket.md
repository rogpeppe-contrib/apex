# Starting a daemon removes a live daemon's socket, leaving the running server unreachable

**Type:** bug
**Area:** daemon / sockets

## Summary

`apex server` (and anything that ends up running it on the same socket path)
removes whatever file is at the socket path and binds a new socket without
first checking whether a daemon is still listening there. The original
daemon keeps running, holding its sessions, windows and processes, but its
socket no longer has a name in the filesystem, so no client can reach it
again. When the second daemon later stops, it removes the path again, so
afterwards there is nothing on the path at all and the first daemon is
still running, orphaned.

## Steps to reproduce

Everything below runs in a private directory (a relative socket path is used
only because the scratch directory's absolute path is longer than `SUN_LEN`).
`HOME` is pointed at a private directory too, so session dumps stay out of
`~/.apex`.

1. `apex -socket=tmp/a.sock -session=first server &`
2. `apex -socket=tmp/a.sock ls` lists `first`.
3. `apex -socket=tmp/a.sock -session=second server &`
4. `apex -socket=tmp/a.sock ls` now lists only `second`. Both daemon
   processes are alive.
5. `apex -socket=tmp/a.sock stop` stops the second daemon. The first is
   still running, but the socket path is gone.

Transcript (`set -x` output; `$W` is the scratch directory):

```
+ apex -socket=tmp/a.sock -session=first server
+ ls -i tmp/a.sock
60840559 tmp/a.sock
+ apex -socket=tmp/a.sock ls
first	0d8c0ed1-e480-439c-bf78-55585297285f
+ apex -socket=tmp/a.sock -session=second server
+ ls -i tmp/a.sock
60841390 tmp/a.sock                      <- different inode: the first daemon's socket was unlinked
+ apex -socket=tmp/a.sock ls
second	155ab47b-57c1-421f-b260-3a88163275bd   <- session "first" can no longer be reached
+ ps -o pid,stat,args -p 1818534,1818605
    PID STAT COMMAND
1818534 Sl   apex -socket=tmp/a.sock -session=first server
1818605 Sl   apex -socket=tmp/a.sock -session=second server
+ ss -xlp
u_str LISTEN 0 4096 tmp/a.sock 86829871 * 0 users:(("apex",pid=1818534,fd=3))
u_str LISTEN 0 4096 tmp/a.sock 86826437 * 0 users:(("apex",pid=1818605,fd=3))
+ ls -l /proc/1818534/fd
lrwx------ 1 rogpeppe rogpeppe 64 Nov 10 13:42 3 -> socket:[86829871]

+ apex -socket=tmp/a.sock stop           <- reaches the second daemon only
+ ps -o pid,stat,args -p 1818534,1818605
    PID STAT COMMAND
1818534 Sl   apex -socket=tmp/a.sock -session=first server   <- still running
+ ls -i tmp/a.sock
ls: cannot access 'tmp/a.sock': No such file or directory
+ apex -socket=tmp/a.sock ls
apex ls: tmp/a.sock: No such file or directory (os error 2)
+ ss -xlp
u_str LISTEN 0 4096 tmp/a.sock 86829871 * 0 users:(("apex",pid=1818534,fd=3))   <- listening, no name
```

## Expected behaviour

Before binding, a starting daemon should try to connect to the existing
socket. If something answers, it should refuse to start and exit with an
error such as `apex server: a daemon is already listening on PATH`. It should
remove the path only when the socket is stale (the connect fails with
`ECONNREFUSED`, meaning nobody is listening). That is the usual way to
handle Unix-domain sockets, and Plan 9's servers behave the same way: a
second `acme`/plumber does not take over the namespace of one that is still
serving. Likewise, a daemon that exits should unlink the path only if it
still names its own socket (for example, compare the dev/inode recorded at
bind time), not whatever another daemon has bound there since.

## Actual behaviour

The second daemon unlinks the first daemon's socket and binds its own. The
first daemon and all its sessions stay alive but cannot be reached by
`apex`, `apex-ui` or anything else. When the second daemon stops, it unlinks
the path again, leaving none.

## Notes

- Environment: Linux, binaries from `target/apex-linux/bin` built from branch
  `001-linux-support` (afce379). Reproduced every time; no GUI needed.
- The unconditional removal is in `Daemon::run_keeping`,
  `crates/apex-server/src/daemon.rs:288-289`:
  ```rust
  let _ = std::fs::remove_file(path);
  let listener = UnixListener::bind(path)?;
  ```
  The doc comment on `Daemon::run` (daemon.rs:264) says "removed first if
  stale", but nothing checks for staleness. The exit path at
  `daemon.rs:351` (`let _ = std::fs::remove_file(path);`) also removes the
  path unconditionally, which is why stopping the second daemon leaves no
  socket.
- The callers try to avoid this, but only on their side and without
  locking: `ensure_server` (`crates/apex-cli/src/main.rs:954`) and
  `shell::ensure_daemon` (`crates/apex-client/src/shell.rs:625`) connect
  first and start a daemon only when the connect fails. `ensure_daemon`'s
  comment describes exactly this failure ("would take its socket and leave
  it running, unreachable"). Running `apex server` directly bypasses those
  checks, and so would two clients racing between the check and
  `spawn_server`. The check belongs in the daemon, at bind time.
- Side effect: both daemons used the same dump directory
  (`~/.apex/sessions/<socket path>/`, from `persist::dir_for`), so two
  daemons on one path also write their session dumps into the same place.
