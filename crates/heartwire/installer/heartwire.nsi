; HeartWire per-user installer. Build with scripts/package.ps1.
; Install folder, uninstall key, shortcuts and startup entry stay fixed, so a
; new version installs over the previous one.

Unicode true
ManifestDPIAware true
SetCompressor /SOLID lzma

!ifndef VERSION
  !error "Pass /DVERSION=x.y.z"
!endif
!ifndef ROOT
  !error "Pass /DROOT=<repository>"
!endif
!ifndef OUTDIR
  !define OUTDIR "${ROOT}\target\release\bundle\nsis"
!endif

!define PRODUCT "HeartWire"
!define PUBLISHER "saitofernando"
!define EXE "heartwire.exe"
!define HOMEPAGE "https://github.com/saitofernando/heartwire"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT}"
!define APP_DIR "${ROOT}\crates\heartwire"

Name "${PRODUCT}"
OutFile "${OUTDIR}\${PRODUCT}_${VERSION}_x64-setup.exe"
InstallDir "$LOCALAPPDATA\${PRODUCT}"
RequestExecutionLevel user
BrandingText "${PRODUCT} ${VERSION}"

VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=1033 "ProductName" "${PRODUCT}"
VIAddVersionKey /LANG=1033 "ProductVersion" "${VERSION}"
VIAddVersionKey /LANG=1033 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=1033 "FileDescription" "${PRODUCT} Setup"
VIAddVersionKey /LANG=1033 "CompanyName" "${PUBLISHER}"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Copyright (c) 2026 ${PUBLISHER}"

!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "LogicLib.nsh"

!define MUI_ICON "${APP_DIR}\icons\icon.ico"
!define MUI_UNICON "${APP_DIR}\icons\icon.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "${APP_DIR}\installer\header.bmp"
!define MUI_HEADERIMAGE_UNBITMAP "${APP_DIR}\installer\header.bmp"
!define MUI_HEADERIMAGE_BITMAP_STRETCH "NoStretchNoCropNoAlign"
!define MUI_HEADERIMAGE_UNBITMAP_STRETCH "NoStretchNoCropNoAlign"
!define MUI_WELCOMEFINISHPAGE_BITMAP "${APP_DIR}\installer\sidebar.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "${APP_DIR}\installer\sidebar.bmp"
!define MUI_BGCOLOR "171B18"
!define MUI_TEXTCOLOR "E7ECE8"
!define MUI_INSTFILESPAGE_COLORS "E7ECE8 202721"
!define MUI_ABORTWARNING

; Welcome (what HeartWire is) → Install → progress → finish with one launch
; checkbox. LICENSE and third-party notices are installed with the app.
!define MUI_WELCOMEPAGE_TITLE "$(WelcomeTitle)"
!define MUI_WELCOMEPAGE_TEXT "$(WelcomeText)"
!define MUI_PAGE_CUSTOMFUNCTION_SHOW WelcomeShow
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_TITLE "$(FinishTitle)"
!define MUI_FINISHPAGE_TEXT "$(FinishText)"
!define MUI_FINISHPAGE_RUN "$INSTDIR\${EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "$(RunText)"
!define MUI_PAGE_CUSTOMFUNCTION_SHOW FinishShow
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "PortugueseBR"
!insertmacro MUI_LANGUAGE "English"

LangString WelcomeTitle ${LANG_PORTUGUESEBR} "Bem-vindo ao HeartWire"
LangString WelcomeTitle ${LANG_ENGLISH} "Welcome to HeartWire"
LangString WelcomeText ${LANG_PORTUGUESEBR} "O HeartWire leva o áudio do Windows para o iPhone pela rede local, com baixa latência, ou usa o celular como microfone do PC.$\r$\n$\r$\nO app fica na bandeja do Windows. No celular, basta ler o QR code: o HeartWire abre no Safari, sem instalar nada da App Store.$\r$\n$\r$\nA instalação é só para o seu usuário e não pede administrador. Código aberto, licença MIT.$\r$\n$\r$\nClique em Instalar para continuar."
LangString WelcomeText ${LANG_ENGLISH} "HeartWire brings Windows audio to your iPhone over the local network with low latency, or uses the phone as a microphone for your PC.$\r$\n$\r$\nThe app lives in the Windows system tray. On the phone, just scan the QR code: HeartWire opens in Safari, with nothing to install from the App Store.$\r$\n$\r$\nIt installs for your user only and does not need administrator rights. Open source, MIT license.$\r$\n$\r$\nClick Install to continue."
LangString FinishTitle ${LANG_PORTUGUESEBR} "HeartWire instalado"
LangString FinishTitle ${LANG_ENGLISH} "HeartWire installed"
LangString FinishText ${LANG_PORTUGUESEBR} "Clique no coração da bandeja para abrir o app."
LangString FinishText ${LANG_ENGLISH} "Click the heart in your system tray to open the app."
LangString RunText ${LANG_PORTUGUESEBR} "Abrir HeartWire"
LangString RunText ${LANG_ENGLISH} "Open HeartWire"

; Ask a running HeartWire to stop audio and exit; anything still running after
; a grace period is closed.
!macro StopRunningApp
  ${If} ${FileExists} "$INSTDIR\${EXE}"
    ExecWait '"$INSTDIR\${EXE}" --quit'
  ${EndIf}
  Sleep 1500
  nsExec::Exec 'taskkill /IM ${EXE} /F'
  Pop $0
  Sleep 500
!macroend

; The welcome page leads straight to installation.
Function WelcomeShow
  GetDlgItem $0 $HWNDPARENT 1
  SendMessage $0 ${WM_SETTEXT} 0 "STR:$(^InstallBtn)"
FunctionEnd

; Themed checkboxes ignore custom text colors; drop the theme so the label
; stays readable on the dark page.
Function FinishShow
  System::Call 'uxtheme::SetWindowTheme(p $mui.FinishPage.Run, w " ", w " ")'
  SetCtlColors $mui.FinishPage.Run E7ECE8 171B18
FunctionEnd

Function .onInit
  SetShellVarContext current
  ; Reuse the folder of an existing installation.
  ReadRegStr $0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
  ${If} $0 != ""
    ; The uninstall entry stores the path in quotes. Strip them before
    ; assigning: NSIS sanitizes $INSTDIR and would also drop the drive colon.
    StrCpy $1 $0 1
    ${If} $1 == '"'
      StrCpy $0 $0 "" 1
      StrCpy $0 $0 -1
    ${EndIf}
    StrCpy $INSTDIR $0
  ${EndIf}
FunctionEnd

Section "HeartWire" Main
  SectionIn RO
  SetOutPath "$INSTDIR"
  !insertmacro StopRunningApp

  File "${ROOT}\target\release\${EXE}"
  File "${ROOT}\LICENSE"
  File "${ROOT}\NOTICE"
  File "${ROOT}\README.md"
  File "${ROOT}\README.en.md"
  File "${ROOT}\CHANGELOG.md"
  File "${ROOT}\CHANGELOG.en.md"
  File "${ROOT}\CONTRIBUTING.md"
  File "${ROOT}\CONTRIBUTING.en.md"
  File "${ROOT}\CREDITS.md"
  File "${ROOT}\CREDITS.en.md"
  File "${ROOT}\SECURITY.md"
  File "${ROOT}\VALIDACAO.md"
  File "${ROOT}\VALIDACAO.en.md"
  File "${ROOT}\THIRD_PARTY_NOTICES.md"
  ; "File /r <folder>" would match same-named folders anywhere under ROOT
  ; (target/, artifacts/); a wildcard inside each folder stays within it.
  RMDir /r "$INSTDIR\licenses"
  RMDir /r "$INSTDIR\docs"
  SetOutPath "$INSTDIR\licenses"
  File /r "${ROOT}\licenses\*.*"
  SetOutPath "$INSTDIR\docs"
  File /r "${ROOT}\docs\*.*"
  SetOutPath "$INSTDIR"

  CreateShortcut "$SMPROGRAMS\${PRODUCT}.lnk" "$INSTDIR\${EXE}"
  CreateShortcut "$DESKTOP\${PRODUCT}.lnk" "$INSTDIR\${EXE}"

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\${PUBLISHER}\${PRODUCT}" "" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${PRODUCT}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" '"$INSTDIR\${EXE}"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "${PUBLISHER}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" '"$INSTDIR"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "MainBinaryName" "${EXE}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "${HOMEPAGE}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "URLUpdateInfo" "${HOMEPAGE}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "HelpLink" "${HOMEPAGE}"
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "EstimatedSize" $0
SectionEnd

Function un.onInit
  SetShellVarContext current
FunctionEnd

Section "Uninstall"
  !insertmacro StopRunningApp
  Delete "$SMPROGRAMS\${PRODUCT}.lnk"
  Delete "$DESKTOP\${PRODUCT}.lnk"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCT}"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" "${PRODUCT}"

  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\*.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\NOTICE"
  RMDir /r "$INSTDIR\licenses"
  RMDir /r "$INSTDIR\docs"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "Software\${PUBLISHER}\${PRODUCT}"
  DeleteRegKey /ifempty HKCU "Software\${PUBLISHER}"
  ; Preferences, the local certificate and logs stay in
  ; %LOCALAPPDATA%\com.heartwire.desktop for a later reinstall.
SectionEnd
