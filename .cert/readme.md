# Self Signed Certificates

In this directory, you will find the self-signed certificates used in the development environment.
You can add it to your system to trust nighly msix packages.

1. Download Seelen.pfx
2. Open a powershell terminal as administrator
4. Go to the directory where you downloaded the file
3. Run the following command
```pwsh
$password = ConvertTo-SecureString -String seelen -Force -AsPlainText
Import-PfxCertificate -FilePath .\Seelen.pfx -CertStoreLocation Cert:\LocalMachine\root -Password $password
```

> [!NOTE]
> These files expire each year and should be replaced with new ones.