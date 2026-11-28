; Poltergeist Windows installer. Build with:
;   ISCC.exe /DAppVersion=0.5.0 packaging\windows\poltergeist.iss
; after `cargo build --release -p poltergeist-app -p poltergeist-cli`.
#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
AppId={{6F1B2C84-5D2E-4B8A-9C57-0A3E1D7F4B90}
AppName=Poltergeist
AppVersion={#AppVersion}
AppPublisher=Sorabbyngo
DefaultDirName={autopf}\Poltergeist
DefaultGroupName=Poltergeist
DisableProgramGroupPage=yes
; Installs for the current user only, so no administrator prompt is needed.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\..\dist
OutputBaseFilename=Poltergeist-Setup-{#AppVersion}
SetupIconFile=..\..\assets\poltergeist.ico
UninstallDisplayIcon={app}\poltergeist-app.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; Flags: unchecked
Name: "startup"; Description: "Start Poltergeist when I sign in to Windows"; Flags: unchecked

[Files]
Source: "..\..\target\release\poltergeist-app.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\release\poltergeist.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Poltergeist"; Filename: "{app}\poltergeist-app.exe"
Name: "{autodesktop}\Poltergeist"; Filename: "{app}\poltergeist-app.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Poltergeist"; ValueData: """{app}\poltergeist-app.exe"""; Tasks: startup; Flags: uninsdeletevalue

[Run]
Filename: "{app}\poltergeist-app.exe"; Description: "Launch Poltergeist"; Flags: nowait postinstall skipifsilent
