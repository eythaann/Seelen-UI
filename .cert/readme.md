# Self Signed Certificates

## Creation

```pwsh
msixherocli.exe newcert --directory .\.cert --name Seelen --password Seelen --subject CN=7E60225C-94CB-4B2E-B17F-0159A11074CB --validUntil "14/12/2026 6:31:10 pm"
```

## Usage

In this directory, you will find the self-signed certificates used in the development environment. You can add it to
your system to trust nighly msix packages.

1. Download Seelen.pfx
2. Open a powershell terminal as administrator
3. Go to the directory where you downloaded the file
4. Run the following command

```pwsh
$password = ConvertTo-SecureString -String seelen -Force -AsPlainText
Import-PfxCertificate -FilePath .\Seelen.pfx -CertStoreLocation Cert:\LocalMachine\root -Password $password
```

> [!NOTE]
> These files expire each year and should be replaced with new ones.
