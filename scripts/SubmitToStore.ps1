Set-ExecutionPolicy RemoteSigned -Force

# Prepare materials from previous steps and runner environment variables
$jsonContent = Get-Content -Path ".\package.json" -Raw
$jsonObject = $jsonContent | ConvertFrom-Json
$version = $jsonObject.version
$appxUploadFilePath = ".\target\release\bundle\msix\Seelen.SeelenUI_${version}_x64__p6yyn03m1894e.msix"

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
New-SubmissionPackage -ConfigPath $configFilePath -AppxPath $appxUploadFilePath -OutPath $sbTempFolderPath -OutName 'submission'

# ********* UPDATE SUBMISSION *********
$submissionDataPath = Join-Path -Path $sbTempFolderPath -ChildPath 'submission.json'
$submissionPackagePath = Join-Path -Path $sbTempFolderPath -ChildPath 'submission.zip'

Update-ApplicationSubmission -Verbose -Force -ReplacePackages -AutoCommit -AppId $appStoreId -SubmissionDataPath $submissionDataPath -PackagePath $submissionPackagePath -NoStatus