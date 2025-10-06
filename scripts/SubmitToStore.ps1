Set-ExecutionPolicy RemoteSigned -Force

# Prepare materials from previous steps and runner environment variables
$jsonContent = Get-Content -Path ".\package.json" -Raw
$jsonObject = $jsonContent | ConvertFrom-Json
$version = $jsonObject.version

# Look for both x64 and arm64 MSIX files
$msixFiles = @(
    ".\target\release\bundle\msix\Seelen.SeelenUI_${version}.0_x64__p6yyn03m1894e.msix",
    ".\target\release\bundle\msix\Seelen.SeelenUI_${version}.0_arm64__p6yyn03m1894e.msix"
)

# Filter existing files
$existingMsixFiles = $msixFiles | Where-Object { Test-Path $_ }

if ($existingMsixFiles.Count -eq 0) {
    Write-Error "No MSIX files found"
    exit 1
}

Write-Host "Found MSIX files:"
$existingMsixFiles | ForEach-Object { Write-Host "  - $_" }

# Use first file as primary, but pass all to New-SubmissionPackage
$appxUploadFilePath = $existingMsixFiles[0]

# $appxUploadFilePath = $PSBoundParameters["appxPathParam"]
$username = $env:PartnerCenterClientId
$password = ConvertTo-SecureString $env:PartnerCenterClientSecret -AsPlainText -Force
$appStoreId = $env:PartnerCenterStoreId
$tenantId = $env:PartnerCenterTenantId

$scriptPath = Split-Path -Path $MyInvocation.MyCommand.Definition -Parent

# ********* Create temporary directory for submission artifacts *********
$sbTempFolderPath = New-Item -Type Directory -Force -Path (Join-Path -Path $scriptPath -ChildPath 'SBTemp')

# ********* Install StoreBroker and import PowerShell Module *********
Set-PSRepository -Name "PSGallery" -InstallationPolicy Trusted
Install-Module -Name StoreBroker

# ********* Authenticate Store Broker *********
$cred = New-Object System.Management.Automation.PSCredential ($username, $password)
Set-StoreBrokerAuthentication -TenantId $tenantId -Credential $cred

# ********* Prepare Submission Package *********
$configFilePath = (Join-Path -Path $scriptPath -ChildPath 'submission.json')

# StoreBroker supports multiple packages - pass all MSIX files as array
if ($existingMsixFiles.Count -eq 1) {
    New-SubmissionPackage -ConfigPath $configFilePath -AppxPath $appxUploadFilePath -OutPath $sbTempFolderPath -OutName 'submission'
} else {
    New-SubmissionPackage -ConfigPath $configFilePath -AppxPath $existingMsixFiles -OutPath $sbTempFolderPath -OutName 'submission'
}

# ********* UPDATE SUBMISSION *********
$submissionDataPath = Join-Path -Path $sbTempFolderPath -ChildPath 'submission.json'
$submissionPackagePath = Join-Path -Path $sbTempFolderPath -ChildPath 'submission.zip'

Update-ApplicationSubmission -Verbose -Force -ReplacePackages -AutoCommit -AppId $appStoreId -SubmissionDataPath $submissionDataPath -PackagePath $submissionPackagePath -NoStatus