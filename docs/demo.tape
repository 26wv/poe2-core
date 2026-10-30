# Render with: vhs docs/demo.tape (not rendered in this repo)
# Shows `poe2 doctor` finding a synthetic native Steam install in an empty temp HOME, with every
# other variable the Linux locator reads unset, so no real path can appear
# (the layout of steam_layout in crates/poe2-cli/tests/common/mod.rs). Needs `poe2` on PATH.

Output docs/demo.gif

Set Shell "bash"
Set Width 1200
Set Height 600

Hide
Type `unset XDG_DATA_HOME XDG_CONFIG_HOME WINEPREFIX POE2_INSTALL_DIR POE2_USER_DIR && export HOME=$(mktemp -d /tmp/poe2-demo.XXXXXX) && S="$HOME/.local/share/Steam/steamapps" && mkdir -p "$S/common/Path of Exile 2/logs" && printf '"AppState" { "appid" "2694490" "installdir" "Path of Exile 2" }' > "$S/appmanifest_2694490.acf" && printf 'x\n' > "$S/common/Path of Exile 2/logs/Client.txt" && clear`
Enter
Show

Type "poe2 doctor"
Enter
Sleep 4s
