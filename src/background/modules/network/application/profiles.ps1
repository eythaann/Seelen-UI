$tempFolder = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tempFolder > $null

netsh wlan export profile folder=$tempFolder key=clear > $null

$jsonOutputFile = [System.IO.Path]::Combine($tempFolder, "wifi_profiles.json")

$wifiProfiles = Get-ChildItem -Path $tempFolder -Filter *.xml | ForEach-Object {
    [xml]$xmlContent = Get-Content -Path $_.FullName
    [PSCustomObject]@{
        ProfileName = $xmlContent.WLANProfile.name
        SSID = $xmlContent.WLANProfile.SSIDConfig.SSID.name
        Authentication = $xmlContent.WLANProfile.MSM.Security.authEncryption.authentication
        Encryption = $xmlContent.WLANProfile.MSM.Security.authEncryption.encryption
        Password = $xmlContent.WLANProfile.MSM.Security.sharedKey.keyMaterial
    }
}

$wifiProfiles | ConvertTo-Json | Set-Content -Path $jsonOutputFile

Write-Host $jsonOutputFile -NoNewline