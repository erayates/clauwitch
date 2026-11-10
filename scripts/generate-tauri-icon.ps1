param(
    [string]$OutputPath = (Join-Path (Split-Path -Parent $PSScriptRoot) 'src-tauri\app-icon.png')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing

function New-ClayTile {
    param([int]$Size)
    $bitmap = New-Object System.Drawing.Bitmap $Size, $Size
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.Clear([System.Drawing.Color]::Transparent)

    $clay = [System.Drawing.Color]::FromArgb(217, 119, 87)
    $brush = New-Object System.Drawing.SolidBrush $clay
    $margin = $Size * 0.012
    $radius = $Size * 0.22
    $diameter = $radius * 2
    $x = $margin
    $y = $margin
    $width = $Size - $margin * 2
    $height = $Size - $margin * 2

    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($x, $y, $diameter, $diameter, 180, 90)
    $path.AddArc($x + $width - $diameter, $y, $diameter, $diameter, 270, 90)
    $path.AddArc($x + $width - $diameter, $y + $height - $diameter, $diameter, $diameter, 0, 90)
    $path.AddArc($x, $y + $height - $diameter, $diameter, $diameter, 90, 90)
    $path.CloseFigure()
    $graphics.FillPath($brush, $path)
    $path.Dispose()
    $brush.Dispose()

    $center = $Size / 2.0
    $radiusArc = $Size * 0.275
    $stroke = [Math]::Max(1.2, $Size * 0.078)
    $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::White), $stroke
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round

    $rect = New-Object System.Drawing.RectangleF (
        $center - $radiusArc), (
        $center - $radiusArc), (
        $radiusArc * 2), (
        $radiusArc * 2)

    $graphics.DrawArc($pen, $rect, 205, 130)
    $graphics.DrawArc($pen, $rect, 25, 130)

    $arrowLength = $stroke * 1.75
    $arrowHalfWidth = $stroke * 0.95

    foreach ($angle in @(335.0, 155.0)) {
        $radians = $angle * [Math]::PI / 180.0
        $cos = [Math]::Cos($radians)
        $sin = [Math]::Sin($radians)
        $endX = $center + $radiusArc * $cos
        $endY = $center + $radiusArc * $sin
        $tangentX = -$sin
        $tangentY = $cos
        $normalX = $cos
        $normalY = $sin

        $points = New-Object System.Drawing.PointF[] 3
        $points[0] = New-Object System.Drawing.PointF (
            $endX + $normalX * $arrowHalfWidth), (
            $endY + $normalY * $arrowHalfWidth)
        $points[1] = New-Object System.Drawing.PointF (
            $endX - $normalX * $arrowHalfWidth), (
            $endY - $normalY * $arrowHalfWidth)
        $points[2] = New-Object System.Drawing.PointF (
            $endX + $tangentX * $arrowLength), (
            $endY + $tangentY * $arrowLength)

        $whiteBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
        $graphics.FillPolygon($whiteBrush, $points)
        $whiteBrush.Dispose()
    }

    $pen.Dispose()
    $graphics.Dispose()
    return $bitmap
}

$sizes = @(16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
$folder = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $folder | Out-Null

$large = New-ClayTile 1024
$large.Save($OutputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$large.Dispose()

$previewFolder = Join-Path $folder 'icon-previews'
New-Item -ItemType Directory -Force -Path $previewFolder | Out-Null

foreach ($size in $sizes) {
    $image = New-ClayTile $size
    $image.Save((Join-Path $previewFolder "icon-$size.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $image.Dispose()
}

$sheetSize = 700
$sheet = New-Object System.Drawing.Bitmap $sheetSize, 260
$sheetGraphics = [System.Drawing.Graphics]::FromImage($sheet)
$sheetGraphics.Clear([System.Drawing.Color]::FromArgb(31, 30, 29))
$left = 20
foreach ($size in @(16, 24, 32, 48, 64, 128)) {
    $image = [System.Drawing.Image]::FromFile((Join-Path $previewFolder "icon-$size.png"))
    $drawSize = [Math]::Min($size, 128)
    if ($size -eq 16 -or $size -eq 24 -or $size -eq 32 -or $size -eq 48) {
        $drawSize = $size * 2
    }
    $sheetGraphics.DrawImage($image, $left, 60, $drawSize, $drawSize)
    $image.Dispose()
    $left += $drawSize + 18
}
$sheetGraphics.Dispose()
$sheet.Save((Join-Path $folder 'icon-preview-sheet.png'), [System.Drawing.Imaging.ImageFormat]::Png)
$sheet.Dispose()

Write-Host "Icon source written to $OutputPath"
