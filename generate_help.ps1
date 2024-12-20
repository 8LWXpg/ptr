Push-Location
Set-Location $PSScriptRoot

$file = '.\README.md'
$content = Get-Content $file -Raw
[regex]::Replace($content, '```(?!pwsh)(.+)\n[\s\S]+?```' , { param([System.Text.RegularExpressions.Match]$match)
		"``````$($match.Groups[1])
$((target\debug\ptr.exe "$($match.Groups[1])".Split(' ')) -join "`n")
``````"
	}) | Out-File -NoNewline -FilePath $file

Pop-Location