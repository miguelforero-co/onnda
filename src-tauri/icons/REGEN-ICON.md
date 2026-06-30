# Regenerar el icono (macOS 26 Tahoe / Liquid Glass)

Fuente editable: `AppIcon.icon/` (formato Icon Composer: `icon.json` + `Assets/`).
- Fondo crema `#d6d8d7`, barras planas (glass off en el grupo).
- Variantes de appearance: dark = fondo oscuro + barras blancas.

## Recompilar el Assets.car tras editar el .icon
```bash
ACTOOL="/Applications/Xcode.app/Contents/Developer/usr/bin/actool"
"$ACTOOL" src-tauri/icons/AppIcon.icon --compile /tmp/carout \
  --app-icon AppIcon --include-all-app-icons --enable-on-demand-resources NO \
  --development-region en --target-device mac \
  --minimum-deployment-target 26.0 --platform macosx \
  --output-partial-info-plist /tmp/carout/partial.plist
cp /tmp/carout/Assets.car src-tauri/icons/Assets.car
```

## Previsualizar sin compilar (render real de Apple)
```bash
ICT="/Applications/Xcode.app/Contents/Applications/Icon Composer.app/Contents/Executables/ictool"
"$ICT" src-tauri/icons/AppIcon.icon --export-image --output-file /tmp/preview.png \
  --platform macOS --rendition Default --width 1024 --height 1024 --scale 1
```
Renditions: `Default`, `Dark`, `TintedLight`, `TintedDark`.

## Cableado (ya hecho)
- `Info.plist`: `CFBundleIconName = AppIcon`
- `tauri.conf.json` → `bundle.resources`: `{ "icons/Assets.car": "Assets.car" }`
- `.icns/.png` clásicos = fallback para macOS < 26.

> ictool (CLI) no renderiza fills custom en dark; la variante dark se verifica en el sistema real.
