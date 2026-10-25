# Render NSIS BMP branding from the existing HeartWire icon. Windows only.
# Run from any directory: pwsh -NoProfile -File scripts/installer-assets.ps1
# No external tools or packages; icon pixels and colors remain unchanged.
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$assetDirectory = Join-Path $projectRoot 'crates\heartwire\installer'
$iconPath = Join-Path $projectRoot 'crates\heartwire\icons\128x128.png'
New-Item -ItemType Directory -Path $assetDirectory -Force | Out-Null

$palette = @{
    Background = [System.Drawing.ColorTranslator]::FromHtml('#171b18')
    Text = [System.Drawing.ColorTranslator]::FromHtml('#e7ece8')
    Muted = [System.Drawing.ColorTranslator]::FromHtml('#a2afa6')
    Mint = [System.Drawing.ColorTranslator]::FromHtml('#79edbb')
}
$icon = [System.Drawing.Image]::FromFile($iconPath)
$textBrush = [System.Drawing.SolidBrush]::new($palette.Text)
$mutedBrush = [System.Drawing.SolidBrush]::new($palette.Muted)
$mintBrush = [System.Drawing.SolidBrush]::new($palette.Mint)
$titleFont = [System.Drawing.Font]::new('Segoe UI', 19, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
$headerFont = [System.Drawing.Font]::new('Segoe UI', 13, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
$smallFont = [System.Drawing.Font]::new('Segoe UI', 11, [System.Drawing.FontStyle]::Regular, [System.Drawing.GraphicsUnit]::Pixel)

function New-BrandBitmap {
    param([int]$Width, [int]$Height)
    $bitmap = [System.Drawing.Bitmap]::new($Width, $Height, [System.Drawing.Imaging.PixelFormat]::Format24bppRgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.Clear($palette.Background)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $graphics.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::ClearTypeGridFit
    return @{ Bitmap = $bitmap; Graphics = $graphics }
}

try {
    $sidebar = New-BrandBitmap -Width 164 -Height 314
    try {
        $sidebar.Graphics.DrawImage($icon, [System.Drawing.Rectangle]::new(34, 48, 96, 96))
        $sidebar.Graphics.DrawString('HeartWire', $titleFont, $textBrush, 24, 169)
        $sidebar.Graphics.DrawString('Windows + iPhone', $smallFont, $mutedBrush, 25, 201)
        $sidebar.Graphics.FillRectangle($mintBrush, 25, 229, 28, 3)
        $sidebar.Graphics.DrawString('saitofernando', $smallFont, $mutedBrush, 25, 277)
        $sidebar.Bitmap.Save((Join-Path $assetDirectory 'sidebar.bmp'), [System.Drawing.Imaging.ImageFormat]::Bmp)
    } finally {
        $sidebar.Graphics.Dispose()
        $sidebar.Bitmap.Dispose()
    }

    $header = New-BrandBitmap -Width 150 -Height 57
    try {
        $header.Graphics.DrawImage($icon, [System.Drawing.Rectangle]::new(8, 12, 32, 32))
        $header.Graphics.DrawString('HeartWire', $headerFont, $textBrush, 46, 19)
        $header.Bitmap.Save((Join-Path $assetDirectory 'header.bmp'), [System.Drawing.Imaging.ImageFormat]::Bmp)
    } finally {
        $header.Graphics.Dispose()
        $header.Bitmap.Dispose()
    }
} finally {
    $icon.Dispose()
    $textBrush.Dispose()
    $mutedBrush.Dispose()
    $mintBrush.Dispose()
    $titleFont.Dispose()
    $headerFont.Dispose()
    $smallFont.Dispose()
}

Write-Output 'NSIS branding: crates/heartwire/installer/{sidebar,header}.bmp'
