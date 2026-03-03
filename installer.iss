[Setup]
; --- IDENTIFICATION ---
AppName=Tablet Driver
AppVersion=1.26.0303.03
AppPublisher=iSweat
OutputBaseFilename=Tablet_Driver_Setup_x64

; --- ARCHITECTURE ---
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; --- INSTALLATION ---
DefaultDirName={commonpf}\TabletDriver
DefaultGroupName=Tablet Driver
UninstallDisplayIcon={app}\tablet_driver.exe
Compression=lzma2
SolidCompression=yes
OutputDir=user_mode_dist

; --- DROITS ET SÉCURITÉ ---
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog

; --- LOGIQUE D'AUTO-UPDATE ---
AppMutex=TabletDriverMutex
CloseApplications=yes
DirExistsWarning=no
SetupMutex=TabletDriverSetupMutex

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\tablet_driver.exe"; DestDir: "{app}"; Flags: ignoreversion restartreplace

[Icons]
Name: "{group}\Tablet Driver"; Filename: "{app}\tablet_driver.exe"
Name: "{commondesktop}\Tablet Driver"; Filename: "{app}\tablet_driver.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\tablet_driver.exe"; Description: "{cm:LaunchProgram,Tablet Driver}"; Flags: nowait postinstall skipifsilent