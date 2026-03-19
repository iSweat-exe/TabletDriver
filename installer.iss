[Setup]
; --- IDENTIFICATION ---
AppName=Next Tablet Driver
AppVersion=1.26.1903.03
AppPublisher=iSweat
OutputBaseFilename=Next_Tablet_Driver_Setup_x64

; --- ARCHITECTURE ---
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

; --- INSTALLATION ---
DefaultDirName={commonpf}\NextTabletDriver
DefaultGroupName=Next Tablet Driver
UninstallDisplayIcon={app}\next_tablet_driver.exe
Compression=lzma2
SolidCompression=yes
OutputDir=user_mode_dist

; --- DROITS ET SÉCURITÉ ---
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog

; --- LOGIQUE D'AUTO-UPDATE ---
AppMutex=NextTabletDriverMutex
CloseApplications=yes
DirExistsWarning=no
SetupMutex=NextTabletDriverSetupMutex

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\next_tablet_driver.exe"; DestDir: "{app}"; Flags: ignoreversion restartreplace

[Icons]
Name: "{group}\Next Tablet Driver"; Filename: "{app}\next_tablet_driver.exe"
Name: "{commondesktop}\Next Tablet Driver"; Filename: "{app}\next_tablet_driver.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\next_tablet_driver.exe"; Description: "{cm:LaunchProgram,Next Tablet Driver}"; Flags: nowait postinstall skipifsilent