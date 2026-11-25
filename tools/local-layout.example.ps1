# Template for tools\local-layout.ps1 — copy it to tools\local-layout.ps1 and edit.
#
# local-layout.ps1 is gitignored, so a local copy never reaches the repository. The tracked
# scripts dot-source it when it exists and fall back to portable defaults otherwise:
#
#   project root   the folder above tools\
#   install dir    %ProgramFiles%\LayerShop
#   cargo target   <project root>\target
#   ComfyUI        http://127.0.0.1:8188, graphs under <project root>\workflows

# Where this checkout lives. Omit to derive it from the scripts' own location.
# $LayoutProjectRoot = 'C:\src\layershop'

# The production install that update.ps1 manages.
# $LayoutInstallDir = 'C:\Program Files\LayerShop'

# Cargo's artifact directory, if you keep it outside the repo.
# $LayoutCargoTarget = 'C:\cargo-target\layershop'

# Local ComfyUI server and the API workflow folder make-config.ps1 reads presets from.
# $LayoutComfyUrl = 'http://127.0.0.1:8188'
# $LayoutGraphDir = 'C:\workflows\api-wks\layershop'
