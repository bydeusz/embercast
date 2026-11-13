import AppKit
import CoreText

let arguments = CommandLine.arguments
let fontURL = URL(fileURLWithPath: arguments[1]) as CFURL
let outputURL = URL(fileURLWithPath: arguments[2])

let size = 1024
let side = CGFloat(size)
let bitmap = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)

func token(_ hex: UInt32, alpha: CGFloat = 1) -> NSColor {
    NSColor(
        srgbRed: CGFloat((hex >> 16) & 0xff) / 255,
        green: CGFloat((hex >> 8) & 0xff) / 255,
        blue: CGFloat(hex & 0xff) / 255,
        alpha: alpha)
}

let tile = NSRect(x: 100, y: 100, width: side - 200, height: side - 200)
token(0x121217).setFill()
NSBezierPath(roundedRect: tile, xRadius: 185, yRadius: 185).fill()

let frame = NSBezierPath(roundedRect: tile.insetBy(dx: 52, dy: 52), xRadius: 136, yRadius: 136)
frame.lineWidth = 6
token(0xd18b2c, alpha: 0.55).setStroke()
frame.stroke()

let descriptors = CTFontManagerCreateFontDescriptorsFromURL(fontURL) as! [CTFontDescriptor]
let weightAxis = 0x7767_6874
let bold = CTFontDescriptorCreateCopyWithAttributes(
    descriptors[0],
    [kCTFontVariationAttribute: [weightAxis: 600]] as CFDictionary)
let font = CTFontCreateWithFontDescriptor(bold, 560, nil) as NSFont

let letter = NSAttributedString(
    string: "E", attributes: [.font: font, .foregroundColor: token(0xd18b2c)])
let line = CTLineCreateWithAttributedString(letter)
let context = NSGraphicsContext.current!.cgContext
let glyph = CTLineGetImageBounds(line, context)
context.textPosition = CGPoint(x: side / 2 - glyph.midX, y: side / 2 - glyph.midY)
CTLineDraw(line, context)

NSGraphicsContext.restoreGraphicsState()
try! bitmap.representation(using: .png, properties: [:])!.write(to: outputURL)
