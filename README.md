# xdg-desktop-portal-logind

A standalone `org.freedesktop.impl.portal.Inhibit` backend to forward sleep inhibition to systemd-logind
and elogind.

| Application request | Result |
|---|---|
| Inhibit Suspend | Acquires a logind sleep/block lock |
| Inhibit Idle | Acquires an idle/block lock; your configured swayidle suppresses its timeouts |
| Inhibit Suspend & Idle | Acquires both |
| Inhibit Logout or User Switching | Fails |
| Inhibit Idle combined with Logout, for example | Entire request fails, including the idle portion |
| Create a session monitor | Fails; no monitoring session is created |
| Receive screensaver/session-ending notifications | Unavailable through this backend |

No `org.freedesktop.ScreenSaver`.

We provide none of these (CreateMonitor is a stub):

- Monitoring session objects.
- Screensaver-active notifications.
- Session-state notifications: Running, Query End, Ending.
- Session-end acknowledgement handling through QueryEndResponse.
- Monitoring-session closure/lifecycle handling.

Monitoring simply fails.

## When use xdg-desktop-portal-logind

There are other implementations of this.  For example, on GNOME, GNOME already does all of this.  In such a case you don't need this project.

There is also a project "xdg-desktop-portal-gtk".  That forwards Idle-only requests to org.freedesktop.ScreenSaver.  It rejects Suspend and combined Suspend&Idle requests. For example, Niri implements org.freedesktop.ScreenSaver.Inhibit and UnInhibit.

So if you use sway or niri or any other compositor *and* you and need system suspend support from within a container, you need this and not `xdg-desktop-portal-gtk`.  We recommend also using swayidle (see below).

## Integration

An (elogind/systemd-enabled) swayidle that initializes logind integration (for example, via `before-sleep`, `lock`, or `idlehint`) suppresses its idle timeouts as well while a system idle inhibitor is held.

## Installation

From the source checkout, first build the daemon, then install:

```sh
cargo build --release --locked
cargo xtask install --binary target/release/xdg-desktop-portal-logind --prefix /usr/local
```

In `~/.config/xdg-desktop-portal/portals.conf`, preserve existing backend preferences
and add (or replace) this key under `[preferred]`:

```ini
[preferred]
# Keep existing default= and other interface preferences here.
org.freedesktop.impl.portal.Inhibit=logind
```
