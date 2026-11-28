# 👻 Poltergeist

**Keeps your computer awake, and can move the mouse or click for you.**
It works offline, needs no account, and sends nothing anywhere.

- **Keep awake:** stops your screen from turning off or locking.
- **Move the cursor:** small, natural movements so the computer never looks idle.
- **Auto-click:** clicks a spot you choose, as often as you choose.

## Download

Pick the file for your device. Then follow the matching steps below.

| Your device | Download |
|---|---|
| **Windows** 10 or 11 | [Poltergeist-Setup.exe](https://github.com/odolocharles/poltergeist/releases/latest/download/Poltergeist-Setup.exe) |
| **Mac** (macOS 11 or newer) | [Poltergeist.dmg](https://github.com/odolocharles/poltergeist/releases/latest/download/Poltergeist.dmg) |
| **Linux** (Fedora, Ubuntu and similar) | [poltergeist-linux-x86_64.run](https://github.com/odolocharles/poltergeist/releases/latest/download/poltergeist-linux-x86_64.run) |
| **Android** 8 or newer | [poltergeist.apk](https://github.com/odolocharles/poltergeist/releases/latest/download/poltergeist.apk) |

All versions are on the [Releases page](https://github.com/odolocharles/poltergeist/releases).

## Install

### Windows

1. Double-click **Poltergeist-Setup.exe**.
2. If Windows shows **"Windows protected your PC"**, click **More info**, then **Run anyway**. This appears because the app is new and not yet signed with a paid certificate. It is safe.
3. Click **Next**, then **Install**, then **Finish**. Poltergeist opens.

You can tick **Start Poltergeist when I sign in** during setup so it is always ready.

### Mac

1. Double-click **Poltergeist.dmg**. A window opens.
2. Drag **Poltergeist** onto the **Applications** folder.
3. Open your **Applications** folder, **right-click** Poltergeist, choose **Open**, then click **Open** again. You only need to do this the first time (the app is not yet notarized by Apple).
4. Click **Start**. When your Mac asks for **Accessibility** access, click **Open System Settings**, switch **Poltergeist** on, then click **Start** again. Without this permission your Mac ignores the mouse and clicks, so Poltergeist can only simulate.

### Linux

You need a recent desktop with GTK 4 and libadwaita. Fedora 40 or newer and Ubuntu 24.04 or newer have them. Open **Terminal** and run:

```bash
cd ~/Downloads
chmod +x poltergeist-linux-x86_64.run
./poltergeist-linux-x86_64.run
```

It installs Poltergeist and adds it to your app menu. It asks once for your password, to allow real mouse and click control. Say **Y**. Then **log out and log in again once**, and open **Poltergeist** from your app menu.

If something is missing, the installer tells you the exact command to run.

### Android

1. On your phone, download **poltergeist.apk** (or download it on a computer and copy it over).
2. Tap the file. If your phone asks, allow installing from that source ("Install unknown apps").
3. Open **Poltergeist** and tap **Keep screen awake**.

On Android, Poltergeist keeps the screen on **while the app is open and in front**. Phones do not allow an app to move a cursor or tap for you, so this version only does keep-awake.

## How to use it

1. Open Poltergeist.
2. Under **What should it do?**, pick **Keep awake**, **Move cursor**, or **Both**.
3. To click automatically, turn on **Click automatically**, then **click inside the preview** to choose where it clicks. Choose how often, and optionally how many clicks before it stops.
4. Press **Start**. Press **Stop** whenever you want it to stop.

Everything you change takes effect straight away and is saved automatically.

**Want to check that clicking really works?** Use the **Test pad**: place the click spot over the pad and watch its counter go up.

**Stay safe with auto-click.** It clicks whatever is under the pointer, so pick a harmless spot first. Setting **Stop after clicks** means it always stops by itself.

## If something goes wrong

**It says "Simulation" and the mouse does not move.**
Poltergeist is not yet allowed to control the mouse. On **Linux**, run the installer again and answer **Y**, then log out and in. On a **Mac**, allow it under System Settings, Privacy & Security, Accessibility. On **Windows** this should not happen; if it does, close and reopen the app.

**Windows or Mac blocks the app when I open it.**
Follow the install steps above. The first-time warning is normal for new apps.

**Clicks do not work inside some Windows programs.**
Windows does not let one app click inside programs running as administrator. Right-click Poltergeist and choose **Run as administrator**.

**My phone screen still turns off.**
Android only keeps the screen on while Poltergeist is open and in front. Leave it on screen.

**The Mac asks for Accessibility again after an update.**
That is normal for now. Switch it on again.

## Start it with your computer

- **Windows:** tick the option during setup, or put a shortcut to Poltergeist in the `shell:startup` folder.
- **Mac:** System Settings, General, Login Items, then **+**, then choose Poltergeist.
- **Linux:** add Poltergeist under **Startup Applications**, or use the background service below.

<details>
<summary>Linux background service (no window needed)</summary>

```bash
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/poltergeist.service << 'EOF'
[Unit]
Description=Poltergeist keep-awake and automation

[Service]
ExecStart=%h/.local/bin/poltergeist run
Restart=on-failure

[Install]
WantedBy=default.target
EOF
systemctl --user enable --now poltergeist
```

It uses the same settings as the app. Stop it with `systemctl --user disable --now poltergeist`.
</details>

<details>
<summary>Command line (optional)</summary>

```bash
poltergeist run --dry-run                              # show what it would do, touch nothing
poltergeist run --click-at 50,50 --click-every 5       # click the middle of the screen every 5 seconds
poltergeist run --mode keep-awake                      # only keep the screen awake
poltergeist config                                     # show your settings and where they are saved
```

`--click-at X,Y` is a position in percent of the screen. Stop with **Ctrl+C**. On Windows the command is `poltergeist.exe`, in the install folder. On a Mac it is `/Applications/Poltergeist.app/Contents/MacOS/poltergeist`.
</details>

## Uninstall

- **Windows:** Settings, Apps, Installed apps, **Poltergeist**, Uninstall.
- **Mac:** drag Poltergeist from Applications to the Bin.
- **Linux:** run `./poltergeist-linux-x86_64.run --uninstall` from the folder you downloaded it to.
- **Android:** press and hold the app icon, then Uninstall.

## Privacy

Poltergeist works entirely on your device. It has **no network access, no account, no tracking and no ads**. Your settings are a small file on your own computer. The Android app does not even ask for internet permission.

## Good to know

- **Windows** marks automated clicks as "injected", and some security software can see that.
- **Mac** needs the Accessibility permission, as described above.
- **Linux** works on X11 and Wayland, and needs the one-time mouse-control permission from the installer.
- **Android** is keep-awake only.

## For developers

Building from source, the command-line tool, how releases are made, and the project layout are in [DEVELOPING.md](DEVELOPING.md).

## License

All rights reserved. © Sorabbyngo.
