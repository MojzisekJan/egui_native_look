"""Compare point geometry and colour-managed renders, without claiming pixel identity."""
import argparse
import io
import json
from pathlib import Path
from PIL import Image, ImageCms, ImageDraw, ImageFont

def srgb(path):
    image = Image.open(path)
    profile = image.info.get('icc_profile')
    if profile:
        return ImageCms.profileToProfile(image, ImageCms.ImageCmsProfile(io.BytesIO(profile)),
                                        ImageCms.createProfile('sRGB'), outputMode='RGB')
    return image.convert('RGB')

def native_image(directory):
    data = json.loads((directory/'reference.json').read_text())
    image = srgb(directory/'reference.png')
    if data['metadata']['platform'] == 'iOS':
        x,y,w,h = data['geometry']['root']
        scale = data['metadata']['scale']
        image = image.crop(tuple(round(v*scale) for v in (x,y,x+w,y+h)))
    return image

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input',type=Path,required=True)
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--baseline',type=Path,help='Optional old mac-light/ios-light egui.png captures')
    args=parser.parse_args();args.out.mkdir(parents=True,exist_ok=True)
    measurements={}
    for directory in sorted(args.input.glob('*')):
        if not (directory/'reference.json').exists() or not (directory/'egui.json').exists(): continue
        native=json.loads((directory/'reference.json').read_text())
        egui=json.loads((directory/'egui.json').read_text())
        measurements[directory.name]={'metadata':native['metadata'],'controls':{}}
        for name,rect in egui['geometry'].items():
            if name not in native['geometry']:continue
            n=native['geometry'][name]
            measurements[directory.name]['controls'][name]={'swiftui_layout':n,'egui_layout':rect,
                'size_delta_points':[round(rect[2]-n[2],3),round(rect[3]-n[3],3)]}
        images=[native_image(directory),srgb(directory/'egui.png')]
        labels=['SwiftUI — native','egui_native_look — updated']
        if args.baseline and directory.name in ['mac-light-regular','ios-light-regular']:
            old=args.baseline/(directory.name.split('-')[0]+'-light')/'egui.png'
            if old.exists():
                images.insert(1,srgb(old));labels.insert(1,'egui_native_look — before')
        width=360;gap=16;top=68
        sheet=Image.new('RGB',(len(images)*width+(len(images)+1)*gap,620+top+16),'#e5e7eb')
        draw=ImageDraw.Draw(sheet)
        font=ImageFont.truetype('/System/Library/Fonts/Supplemental/Arial.ttf',14)
        draw.text((gap,12),directory.name,fill='#111827',font=font)
        for i,(image,label) in enumerate(zip(images,labels)):
            x=gap+i*(width+gap);draw.text((x,40),label,fill='#111827',font=font)
            sheet.paste(image.resize((360,620),Image.Resampling.LANCZOS),(x,top))
        sheet.save(args.out/(directory.name+'.png'))
    (args.out/'geometry.json').write_text(json.dumps(measurements,indent=2))
    print('Saved colour-managed comparisons and geometry deltas.')

if __name__=='__main__':main()
