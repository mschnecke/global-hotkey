$ErrorActionPreference = 'Stop'

$packageName = 'global-hotkey'
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

$packageArgs = @{
  packageName    = $packageName
  fileType       = 'msi'
  url64bit       = 'https://github.com/mschnecke/global-hotkey/releases/download/v0.1.0/Global.Hotkey_0.1.0_x64_en-US.msi'
  softwareName   = 'Global Hotkey*'
  checksum64     = 'REPLACE_WITH_ACTUAL_CHECKSUM'
  checksumType64 = 'sha256'
  silentArgs     = '/qn /norestart'
  validExitCodes = @(0, 3010, 1641)
}

Install-ChocolateyPackage @packageArgs
