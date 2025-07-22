function Show-Tree {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory, ValueFromRemainingArguments)]
        [string[]]$Path
    )

    # -------------------------------------------------------------------- #
    function _Draw {
        param (
            [string]$Node,
            [string]$Indent = ''
        )

        # Get children, ignore “target” dirs, sort: folders first, then files
        $children = Get-ChildItem -LiteralPath $Node -Force -Exclude 'target' |
                    Sort-Object @{ Expression = { -not $_.PSIsContainer } }, Name

        for ($i = 0; $i -lt $children.Count; $i++) {
            $child   = $children[$i]
            $isLast  = $i -eq ($children.Count - 1)
            $branch  = if ($isLast) { '└── ' } else { '├── ' }

            Write-Host "$Indent$branch$($child.Name)"

            if ($child.PSIsContainer) {
                $nextIndent = $Indent + $(if ($isLast) { '    ' } else { '│   ' })
                _Draw -Node $child.FullName -Indent $nextIndent
            }
        }
    }
    # -------------------------------------------------------------------- #

    foreach ($root in $Path) {
        if (-not (Test-Path $root)) {
            Write-Warning "Path not found: $root"
            continue
        }
        $resolved = (Resolve-Path $root).Path
        Write-Host $resolved
        _Draw -Node $resolved
        Write-Host   # blank line between roots
    }
}
