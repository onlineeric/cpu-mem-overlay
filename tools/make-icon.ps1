# Generates assets/cpu-mem-overlay.ico from scratch using System.Drawing.
#
# Run once to (re)generate the icon. The committed .ico is what the build
# script (build.rs) embeds into the .exe; this script does not run as part
# of `cargo build`.
#
# Usage (from repo root):
#   pwsh -NoProfile -File tools/make-icon.ps1
#
# Output: assets/cpu-mem-overlay.ico (multi-image: 16x16, 32x32, 48x48, 256x256)

[CmdletBinding()]
param(
    [string]$OutputPath = (Join-Path $PSScriptRoot '..\assets\cpu-mem-overlay.ico')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

# Resolve absolute output path (parent must exist)
$outputDir = Split-Path -Parent $OutputPath
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}
$outputAbs = [System.IO.Path]::GetFullPath($OutputPath)

function New-IconPng {
    param([int]$Size)

    $bmp = New-Object System.Drawing.Bitmap $Size, $Size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.Clear([System.Drawing.Color]::Transparent)

    # Layout (proportional to Size).
    $pad = [Math]::Max(1, [int]($Size * 0.05))
    $radius = [Math]::Max(2, [int]($Size * 0.18))
    $rectX = $pad
    $rectY = $pad
    $rectW = $Size - 2 * $pad
    $rectH = $Size - 2 * $pad

    # Background: rounded dark slate tile.
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($rectX, $rectY, $radius, $radius, 180, 90)
    $path.AddArc($rectX + $rectW - $radius, $rectY, $radius, $radius, 270, 90)
    $path.AddArc($rectX + $rectW - $radius, $rectY + $rectH - $radius, $radius, $radius, 0, 90)
    $path.AddArc($rectX, $rectY + $rectH - $radius, $radius, $radius, 90, 90)
    $path.CloseFigure()
    $bgBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(240, 28, 32, 40))
    $g.FillPath($bgBrush, $path)

    # Bar geometry: two stacked horizontal bars (CPU on top, MEM on bottom).
    $barInsetX = [int]($Size * 0.18)
    $barInsetY = [int]($Size * 0.28)
    $barX = $rectX + $barInsetX
    $barW = $rectW - 2 * $barInsetX
    $barH = [Math]::Max(2, [int]($Size * 0.16))
    $gap = [Math]::Max(2, [int]($Size * 0.10))
    $cpuY = $rectY + $barInsetY
    $memY = $cpuY + $barH + $gap

    # Track (background) for each bar.
    $trackBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 60, 64, 76))
    $g.FillRectangle($trackBrush, $barX, $cpuY, $barW, $barH)
    $g.FillRectangle($trackBrush, $barX, $memY, $barW, $barH)

    # CPU fill: green at ~70%.
    $cpuFill = [int]([Math]::Round($barW * 0.7))
    $cpuBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 92, 184, 92))
    $g.FillRectangle($cpuBrush, $barX, $cpuY, $cpuFill, $barH)

    # MEM fill: amber at ~45%.
    $memFill = [int]([Math]::Round($barW * 0.45))
    $memBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 240, 173, 78))
    $g.FillRectangle($memBrush, $barX, $memY, $memFill, $barH)

    $g.Dispose()
    $trackBrush.Dispose()
    $cpuBrush.Dispose()
    $memBrush.Dispose()
    $bgBrush.Dispose()
    $path.Dispose()

    # Encode the bitmap as PNG into a byte array (PNG retains alpha; ICO
    # supports PNG-encoded entries since Vista).
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $ms.ToArray()
    $ms.Dispose()
    $bmp.Dispose()

    return ,$bytes
}

# Generate one PNG per icon size.
$sizes = @(16, 32, 48, 256)
$pngs = @{}
foreach ($s in $sizes) {
    $pngs[$s] = New-IconPng -Size $s
}

# Assemble the multi-image ICO file.
#   ICONDIR (6 bytes)
#   ICONDIRENTRY (16 bytes) * N
#   PNG payloads, packed back-to-back.
$icoStream = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter $icoStream
$bw.Write([uint16]0)                # reserved
$bw.Write([uint16]1)                # type = 1 (icon)
$bw.Write([uint16]$sizes.Count)     # image count

# Compute offsets up-front (they sit after ICONDIR + all ICONDIRENTRYs).
$offset = 6 + 16 * $sizes.Count
$entryOffsets = @{}
foreach ($s in $sizes) {
    $entryOffsets[$s] = $offset
    $offset += $pngs[$s].Length
}

# Write each ICONDIRENTRY.
foreach ($s in $sizes) {
    $widthByte  = if ($s -ge 256) { 0 } else { [byte]$s }
    $heightByte = if ($s -ge 256) { 0 } else { [byte]$s }
    $bw.Write([byte]$widthByte)             # width  (0 == 256)
    $bw.Write([byte]$heightByte)            # height (0 == 256)
    $bw.Write([byte]0)                      # color count
    $bw.Write([byte]0)                      # reserved
    $bw.Write([uint16]1)                    # planes
    $bw.Write([uint16]32)                   # bit count
    $bw.Write([uint32]$pngs[$s].Length)     # bytes in res
    $bw.Write([uint32]$entryOffsets[$s])    # offset to image data
}

# Write each PNG payload.
foreach ($s in $sizes) {
    $bw.Write($pngs[$s])
}
$bw.Flush()

[System.IO.File]::WriteAllBytes($outputAbs, $icoStream.ToArray())
$bw.Dispose()
$icoStream.Dispose()

Write-Host "Wrote $outputAbs ($([System.IO.File]::OpenRead($outputAbs).Length) bytes, $($sizes.Count) images: $($sizes -join ', '))"
