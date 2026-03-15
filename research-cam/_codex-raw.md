OpenAI Codex v0.107.0 (research preview)
--------
workdir: /Users/finn.carlsvi/thetta
model: gpt-5.3-codex
provider: openai
approval: never
sandbox: danger-full-access
reasoning effort: none
reasoning summaries: none
session id: 019cd4e1-2459-7d31-8caf-8deb29132a2e
--------
user
You are researching open-source CAM (Computer Aided Manufacturing) software for integration into a Rust-based CNC nesting application. The application already handles 2D nesting optimization of parts onto sheets, and now needs CAM capabilities as a post-processing step.

The specific CAM operations needed are:
1. **Profile cuts** - 2D contour cutting to cut out nested pieces from sheet material
2. **Tab placement** - Calculating where holding tabs should be placed around profile cuts so that cut-out pieces don't move during machining
3. **Pocket cuts** - Clearing material from enclosed areas

This is for a **3-axis CNC router/mill only** - no 2.5D surfacing, no 3D contouring, no multi-axis work.

For EACH of the following 12 open-source CAM projects, provide a detailed analysis covering:

### Per-project analysis structure:
1. **Project overview**: What it is, primary language, license, GitHub/source URL
2. **Activity & maturity**: Last commit date, number of contributors, release history, community size
3. **Profile cut support**: Does it support 2D contour/profile cutting? How does it generate toolpaths for cutting out shapes? Inside vs outside contour support? Climb vs conventional milling options?
4. **Tab support**: Does it support holding tabs? How are tabs placed (automatic vs manual)? Tab size/shape configuration? How sophisticated is the tab placement algorithm?
5. **Pocket cut support**: Does it support pocket clearing? What strategies (zigzag, spiral, offset/contour)? Adaptive/trochoidal clearing?
6. **G-code output**: What G-code dialects/post-processors does it support? How customizable is the output?
7. **Integration potential**: Could this be used as a library (not just a GUI app)? Is there an API? Could the algorithms be extracted? What would it take to integrate into a Rust application?
8. **Strengths for our use case**
9. **Weaknesses for our use case**
10. **Overall rating** (1-5) for suitability to our specific needs (profile cuts, tabs, pockets on 3-axis CNC, integration into a Rust app)

### The 12 projects to analyze:

1. **FreeCAD Path Workbench** - https://github.com/FreeCAD/FreeCAD (Path workbench specifically)
2. **BlenderCAM** - https://github.com/vilemduha/blendercam
3. **PyCAM** - https://github.com/SebKuworworworwor/pycam (or pycam.sourceforge.net)
4. **OpenCAMLib** - https://github.com/aewallin/opencamlib
5. **DerpCAM** - https://github.com/kfoltman/DerpCAM
6. **Kiri:Moto** - https://github.com/GridSpace/grid-apps
7. **jscut** - https://github.com/nicholascross/jscut (or jscut.org)
8. **OpenBuilds CAM** - https://github.com/OpenBuilds/OpenBuilds-CAM
9. **CAMotics** - https://github.com/CauldronDevelopmentLLC/CAMotics
10. **HeeksCNC** - https://github.com/Heeks/heekscnc
11. **DXF2GCODE** - https://github.com/vincentFrancworworais/dxf2gcode (or dxf2gcode.sourceforge.net)
12. **PathCAM** - https://github.com/xenovacivus/PathCAM

Be thorough and specific. If you don't know details about a project, say so rather than guessing. Focus on practical integration considerations.
mcp startup: no servers

thinking
**Planning efficient GitHub data gathering**
codex
I’ll compile this as a source-backed comparison with current project activity and feature specifics for all 12 repositories. First I’m pulling live repository metadata (activity, contributors, releases) and then I’ll verify CAM feature support (profiles/tabs/pockets/G-code/API) from docs and source files for each project.
exec
/bin/zsh -lc 'set -euo pipefail
repos=(
  FreeCAD/FreeCAD
  vilemduha/blendercam
  SebKuzminsky/pycam
  aewallin/opencamlib
  kfoltman/DerpCAM
  GridSpace/grid-apps
  nicholascross/jscut
  OpenBuilds/OpenBuilds-CAM
  CauldronDevelopmentLLC/CAMotics
  Heeks/heekscnc
  dxf2gcode/dxf2gcode
  xenovacivus/PathCAM
)
for r in "${repos[@]}"; do
  echo "=== $r ==="
  curl -sL "https://api.github.com/repos/$r" | jq -r '"'[.full_name,.html_url,.description,.language,.license.spdx_id,.stargazers_count,.forks_count,.open_issues_count,.watchers_count,.default_branch,.pushed_at,.created_at,.updated_at] | @tsv' || true
  echo \"contributors:\" "'$(curl -sI "https://api.github.com/repos/$r/contributors?per_page=1&anon=1" | tr -d '"'\\r' | awk -F'[:, ]+' '/"'^link:/{for(i=1;i<=NF;i++) if($i ~ /page=/){gsub(/.*page=/,"",$i); gsub(/>.*/,"",$i); p=$i}} END{if(p=="") p=1; print p}'"')
  echo \"releases:\" "'$(curl -s "https://api.github.com/repos/$r/releases?per_page=100" | jq '"'length')
  echo
 done" in /Users/finn.carlsvi/thetta succeeded in 16.84s:
FreeCAD/FreeCAD	https://github.com/FreeCAD/FreeCAD	Official source code of FreeCAD, a free and opensource multiplatform 3D parametric modeler.	C++	LGPL-2.1	28960	5247	3770	28960	main	2026-03-09T21:12:35Z	2012-09-09T08:25:02Z	2026-03-09T22:58:56Z
contributors: 960
releases: 84

=== vilemduha/blendercam ===
vilemduha/blendercam	https://github.com/vilemduha/blendercam	Automatically exported from code.google.com/p/blendercam	G-code	GPL-3.0	553	132	22	553	master	2026-01-01T00:44:47Z	2015-03-13T07:32:59Z	2026-03-03T22:07:50Z
contributors: 51
releases: 8

=== SebKuzminsky/pycam ===
SebKuzminsky/pycam	https://github.com/SebKuzminsky/pycam		Python	NOASSERTION	391	123	69	391	master	2024-01-30T06:43:04Z	2017-02-15T16:47:16Z	2026-03-02T03:18:24Z
contributors: 23
releases: 6

=== aewallin/opencamlib ===
aewallin/opencamlib	https://github.com/aewallin/opencamlib	open source computer aided manufacturing algorithms library	C++	LGPL-2.1	496	164	23	496	master	2025-02-12T08:23:45Z	2011-07-14T11:59:42Z	2026-03-06T19:09:30Z
contributors: 31
releases: 6

=== kfoltman/DerpCAM ===

thinking
**Investigating repo validity and sourcing**
🌐 Searching the web...
🌐 Searched: GitHub jscut repository nicholascross jscut

thinking
**Planning repo inspection approach**
codex
I found a couple of repository name mismatches in your list (`jscut` and `dxf2gcode`), so I’m resolving canonical upstream repos first to avoid analyzing stale forks. Next I’ll extract feature evidence (tabs/pocket/profile/posts) directly from project docs and source.
exec
/bin/zsh -lc 'set -euo pipefail
for q in jscut dxf2gcode pycam; do
  echo "=== search $q ==="
  curl -s "https://api.github.com/search/repositories?q=${q}+in:name&sort=stars&order=desc&per_page=10" | jq -r '"'.items[] | [.full_name,.stargazers_count,.html_url] | @tsv' | head -n 10
  echo
done" in /Users/finn.carlsvi/thetta succeeded in 2.34s:
tbfleming/jscut	327	https://github.com/tbfleming/jscut
sotirios-gkountelitsas/jscuts	4	https://github.com/sotirios-gkountelitsas/jscuts
haliver/jscut	1	https://github.com/haliver/jscut
CloudKidStudio/CreateJSCutscene	1	https://github.com/CloudKidStudio/CreateJSCutscene
chilipeppr/widget-jscut	0	https://github.com/chilipeppr/widget-jscut
fedorenko-dmitriy/jscut	0	https://github.com/fedorenko-dmitriy/jscut
folaareago/JScut	0	https://github.com/folaareago/JScut
wrueth/JSCUT	0	https://github.com/wrueth/JSCUT
junlishen/jscutimage	0	https://github.com/junlishen/jscutimage
tiagomab99/nx-jscutlery	0	https://github.com/tiagomab99/nx-jscutlery

=== search dxf2gcode ===
workflo/dxf2gcode	45	https://github.com/workflo/dxf2gcode
cnc-club/dxf2gcode	13	https://github.com/cnc-club/dxf2gcode
ltspicer/dxf2gcode.linux.installer	6	https://github.com/ltspicer/dxf2gcode.linux.installer
billhart/dxf2gcodecli	5	https://github.com/billhart/dxf2gcodecli
ltspicer/dxf2gcode	4	https://github.com/ltspicer/dxf2gcode
SanZamoyski/dxf2gcode-for-grbl	4	https://github.com/SanZamoyski/dxf2gcode-for-grbl
Poofjunior/dxf2gcode	3	https://github.com/Poofjunior/dxf2gcode
oryxr/dxf2gcode	3	https://github.com/oryxr/dxf2gcode

thinking
**Planning canonical repo analysis**
exec
/bin/zsh -lc 'set -euo pipefail
repos=(
  FreeCAD/FreeCAD
  vilemduha/blendercam
  SebKuzminsky/pycam
  aewallin/opencamlib
  kfoltman/DerpCAM
  GridSpace/grid-apps
  tbfleming/jscut
  OpenBuilds/OpenBuilds-CAM
  CauldronDevelopmentLLC/CAMotics
  Heeks/heekscnc
  workflo/dxf2gcode
  xenovacivus/PathCAM
)
for r in "${repos[@]}"; do
  echo "=== $r ==="
  curl -s "https://api.github.com/repos/$r" | jq -r '"'\"url=\\(.html_url)\\nlang=\\(.language) license=\\(.license.spdx_id // \"UNKNOWN\") stars=\\(.stargazers_count) forks=\\(.forks_count) open_issues=\\(.open_issues_count)\\nlast_push=\\(.pushed_at) updated=\\(.updated_at)\\n\"'
  # contributors count from Link header
  c="'$(curl -sI "https://api.github.com/repos/$r/contributors?per_page=1&anon=1" | tr -d '"'\\r' | awk 'BEGIN{p=1} /"'^link:/{n=split($0,a,","); for(i=1;i<=n;i++){ if(a[i] ~ /rel="last"/){ match(a[i],/page=[0-9]+/); if(RLENGTH>0){p=substr(a[i],RSTART+5,RLENGTH-5)} } }} END{print p}'"')
  echo \"contributors="'$c"
  rel_json=$(curl -s "https://api.github.com/repos/$r/releases?per_page=1")
  rel_count=$(curl -s "https://api.github.com/repos/$r/releases?per_page=100" | jq '"'length')
  rel_date="'$(echo "$rel_json" | jq -r '"'if length>0 then .[0].published_at else \"none\" end')
  rel_tag="'$(echo "$rel_json" | jq -r '"'if length>0 then .[0].tag_name else \"none\" end')
  echo \"releases="'$rel_count latest_release=$rel_tag @$rel_date"
  echo
done' in /Users/finn.carlsvi/thetta exited 5 in 3.33s:
url=https://github.com/FreeCAD/FreeCAD
lang=C++ license=LGPL-2.1 stars=28960 forks=5247 open_issues=3770
last_push=2026-03-09T21:12:35Z updated=2026-03-09T22:58:56Z

contributors=1
jq: parse error: Invalid string: control characters from U+0000 through U+001F must be escaped at line 635, column 1

thinking
**Refining GitHub data retrieval**
codex
I hit a parsing issue while bulk-querying release metadata, so I’m switching to per-repo queries and continuing with feature extraction from READMEs/docs. I’ll keep the activity numbers source-backed and then map capabilities to your three operations.
exec
/bin/zsh -lc 'set -euo pipefail
repos=(
  FreeCAD/FreeCAD
  vilemduha/blendercam
  SebKuzminsky/pycam
  aewallin/opencamlib
  kfoltman/DerpCAM
  GridSpace/grid-apps
  tbfleming/jscut
  OpenBuilds/OpenBuilds-CAM
  CauldronDevelopmentLLC/CAMotics
  Heeks/heekscnc
  workflo/dxf2gcode
  xenovacivus/PathCAM
)
for r in "${repos[@]}"; do
  echo "=== $r ==="
  curl -s "https://api.github.com/repos/$r/readme" | jq -r '"'.content // empty' | base64 --decode > /tmp/readme.txt || true
  wc -c /tmp/readme.txt | awk '{print \"readme_bytes=\" "'$1}'"'
  rg -i \"tab|tabs|pocket|profile|contour|inside|outside|climb|conventional|trocho|adaptive|g-code|gcode|post|postprocessor|api|library|plugin\" /tmp/readme.txt | head -n 30 || true
  echo
done" in /Users/finn.carlsvi/thetta succeeded in 5.38s:
readme_bytes=5105
    * **Coin3D library** Open Inventor-compliant 3D scene representation model
    * **Python** FreeCAD offers a broad Python API
Precompiled packages for stable releases are available for Windows, macOS and Linux on the
- Consider posting to the [Forum](https://forum.freecad.org), [Discord](https://discord.com/invite/w2cTKGzccC) channel, or [Reddit](https://www.reddit.com/r/FreeCAD) to verify the issue; 
- Use the most updated stable or [development versions](https://github.com/FreeCAD/FreeCAD/releases/) of FreeCAD; 
- Post version info from `Help > About FreeCAD > Copy to clipboard`; 
- Post a Step-By-Step explanation on how to recreate the issue; 

=== vilemduha/blendercam ===
readme_bytes=8349
### [About](#-about) • [Guide](#-user-guide) • [Features](#-features) • [Post-Processors](#-post-processors) • [Files](#-files-organisation) • [Contribute](#-contribute) • [License](#-license) • [Disclaimer](#-disclaimer)
[**FabexCNC**](https://blendercam.com/) is an open source solution for CAM (*Computer Aided Machining*) toolpath generation and Gcode export.
> *You can also view the full [User, Developer and API Documentation](https://spectralvectors.github.io/blendercam/index.html)*
| **2D & 3D Milling Strategies** | _Profile, Pocket, Drill, Parallel, Cross, Block, Spiral, Medial Axis and more_ | ✅ |
| **Automatic Bridges** | _One click to add Bridges/Tabs to keep your work in place during Cutout Operations_ | ✅ |
| **Chain Operations** | _Combine multiple CAM operations into a Chain, Export as Gcode, or create a Chain Simulation_ | ✅ |
| **Adaptive Milling Speed** | _Adjusts the operation feedrate relative to the chipload calculation_ | ✅ |
## 💻 Post-processors
docs/ - # User, Developer and API docs

thinking
**Planning detailed doc scraping**
exec
/bin/zsh -lc "curl -s 'https://api.github.com/repos/FreeCAD/FreeCAD/contents/src/Mod/Path/PathScripts?ref=main' | jq -r '.[].name' | head -n 200" in /Users/finn.carlsvi/thetta succeeded in 268ms:
jq: error (at <stdin>:5): Cannot index string with string "name"

thinking
**Logging raw API response**
exec
/bin/zsh -lc "curl -s 'https://api.github.com/repos/FreeCAD/FreeCAD/contents/src/Mod/Path/PathScripts?ref=main' | head -n 40" in /Users/finn.carlsvi/thetta succeeded in 240ms:
{
  "message": "Not Found",
  "documentation_url": "https://docs.github.com/rest/repos/contents#get-repository-content",
  "status": "404"
}

thinking
**Planning sparse clone for inspection**
codex
I’m now pulling targeted source paths (not full repos) to verify feature-level claims like tab dressups and pocket/profile parameterization. This avoids guesswork for options like inside/outside and cut direction.
exec
/bin/zsh -lc 'set -euo pipefail
mkdir -p /tmp/cam_research && cd /tmp/cam_research
if [ ! -d FreeCAD ]; then
  git clone --depth 1 --filter=blob:none --sparse https://github.com/FreeCAD/FreeCAD.git FreeCAD >/dev/null 2>&1
  cd FreeCAD
  git sparse-checkout set src/Mod/Path src/Mod/CAM >/dev/null 2>&1 || true
else
  cd FreeCAD
fi
rg --files src/Mod | rg '"'PathDressup|Pocket|Profile|Tag|PathOp|Post|Toolpath|Dressup|Contour|Adaptive|Tab' | head -n 120" in /Users/finn.carlsvi/thetta succeeded in 4.44s:
src/Mod/CAM/CAMTests/TestCentroidLegacyPost.py
src/Mod/CAM/CAMTests/TestFanucPost.py
src/Mod/CAM/CAMTests/TestPathDressupHoldingTags.py
src/Mod/CAM/CAMTests/TestPostGCodes.py
src/Mod/CAM/CAMTests/TestPathOpUtil.py
src/Mod/CAM/CAMTests/TestSnapmakerPost.py
src/Mod/CAM/CAMTests/TestPathDressupArray.py
src/Mod/CAM/CAMTests/TestGrblPost.py
src/Mod/CAM/CAMTests/TestGrblLegacyPost.py
src/Mod/CAM/CAMTests/TestMassoG3Post.py
src/Mod/CAM/CAMTests/TestLinuxCNCLegacyPost.py
src/Mod/CAM/CAMTests/TestTestPost.py
src/Mod/CAM/CAMTests/TestPostMCodes.py
src/Mod/CAM/CAMTests/TestPathAdaptive.py
src/Mod/CAM/CAMTests/TestDressupPost.py
src/Mod/CAM/CAMTests/TestMach3Mach4LegacyPost.py
src/Mod/CAM/CAMTests/PostTestMocks.py
src/Mod/CAM/CAMTests/TestPathDressupDogboneII.py
src/Mod/CAM/CAMTests/TestPathOpDeburr.py
src/Mod/CAM/CAMTests/TestPathPost.py

thinking
**Searching module for parameters**
exec
/bin/zsh -lc "cd /tmp/cam_research/FreeCAD
printf '--- Profile params ---\\n'; rg -n \"Inside|Outside|Side|CutSide|Conventional|Climb|Milling|Direction|Compensation\" src/Mod/CAM/Path/Op/Profile.py | head -n 60
printf '\\n--- Tabs params ---\\n'; rg -n \"auto|manual|tag|tabs|width|height|angle|spacing|count|placement\" src/Mod/CAM/Path/Dressup/Tags.py src/Mod/CAM/Path/Dressup/Gui/Tags.py | head -n 120
printf '\\n--- Pocket strategies ---\\n'; rg -n \"spiral|zig|offset|adaptive|trocho|strategy|pattern|helix|clear\" src/Mod/CAM/Path/Op/Pocket.py src/Mod/CAM/Path/Op/PocketShape.py src/Mod/CAM/Path/Op/Adaptive.py | head -n 120" in /Users/finn.carlsvi/thetta succeeded in 51ms:
--- Profile params ---
34:from PySide.QtCore import QT_TRANSLATE_NOOP
102:                "Direction",
164:                "Side",
166:                QT_TRANSLATE_NOOP("App::Property", "Side of edge that tool should cut"),
173:                    "App::Property", "Make True, if using Cutter Radius Compensation"
220:            "Direction": [
233:            "Side": [
234:                (translate("PathProfile", "Outside"), "Outside"),
235:                (translate("PathProfile", "Inside"), "Inside"),
258:            "Direction": "CW",
263:            "Side": "Outside",
314:        obj.setEditorMode("Side", side)
361:        if obj.Side == "Inside":
395:            direction = "CW" if obj.Direction == "CCW" else "CCW"
397:            direction = obj.Direction
815:            self.cutSide = "E"
816:            self.cutSideTags = eTAG
820:            self.cutSide = "I"
821:            self.cutSideTags = iTAG

thinking
**Planning multi-repo cloning strategy**
exec
/bin/zsh -lc "set -euo pipefail
cd /tmp/cam_research
repos=(
  'vilemduha/blendercam blendercam'
  'SebKuzminsky/pycam pycam'
  'aewallin/opencamlib opencamlib'
  'kfoltman/DerpCAM DerpCAM'
  'GridSpace/grid-apps grid-apps'
  'tbfleming/jscut jscut'
  'OpenBuilds/OpenBuilds-CAM OpenBuilds-CAM'
  'CauldronDevelopmentLLC/CAMotics CAMotics'
  'Heeks/heekscnc heekscnc'
  'workflo/dxf2gcode dxf2gcode'
  'xenovacivus/PathCAM PathCAM'
)
for entry in \""'${repos[@]}"; do
  set -- $entry
  repo=$1; dir=$2
  if [ ! -d "$dir" ]; then
    git clone --depth 1 "https://github.com/$repo.git" "$dir" >/dev/null 2>&1 || echo "clone failed $repo"
  fi
  echo "cloned $dir"
done

echo '"'done'" in /Users/finn.carlsvi/thetta exited 1 in 51ms:
zsh:18: 2: parameter not set

thinking
**Simplifying array handling to fix script failure**
exec
/bin/zsh -lc 'cd /tmp/cam_research
clone(){ repo="$1"; dir="$2"; if [ ! -d "$dir" ]; then git clone --depth 1 "https://github.com/$repo.git" "$dir" >/dev/null 2>&1 || echo "clone failed $repo"; fi; echo "ready $dir"; }
clone vilemduha/blendercam blendercam
clone SebKuzminsky/pycam pycam
clone aewallin/opencamlib opencamlib
clone kfoltman/DerpCAM DerpCAM
clone GridSpace/grid-apps grid-apps
clone tbfleming/jscut jscut
clone OpenBuilds/OpenBuilds-CAM OpenBuilds-CAM
clone CauldronDevelopmentLLC/CAMotics CAMotics
clone Heeks/heekscnc heekscnc
clone workflo/dxf2gcode dxf2gcode
clone xenovacivus/PathCAM PathCAM' in /Users/finn.carlsvi/thetta succeeded in 32.23s:
ready blendercam
ready pycam
ready opencamlib
ready DerpCAM
ready grid-apps
ready jscut
ready OpenBuilds-CAM
ready CAMotics
ready heekscnc
clone failed workflo/dxf2gcode
ready dxf2gcode
ready PathCAM

thinking
**Monitoring ongoing process**

thinking
**Verifying directory existence and case sensitivity**
exec
/bin/zsh -lc 'ls -1 /tmp/cam_research' in /Users/finn.carlsvi/thetta succeeded in 52ms:
CAMotics
DerpCAM
FreeCAD
OpenBuilds-CAM
PathCAM
blendercam
dxf2gcode
grid-apps
heekscnc
jscut
opencamlib
pycam

thinking
**Checking empty directory presence**
exec
/bin/zsh -lc 'for d in dxf2gcode jscut; do echo "== $d =="; cd /tmp/cam_research/$d 2>/dev/null || true; git remote -v || true; ls -la | head; done' in /Users/finn.carlsvi/thetta succeeded in 52ms:
== dxf2gcode ==
origin	https://github.com/workflo/dxf2gcode.git (fetch)
origin	https://github.com/workflo/dxf2gcode.git (push)
total 88
drwxr-x---@ 11 finn.carlsvi  wheel    352 Mar  9 16:16 .
drwxr-x---@ 14 finn.carlsvi  wheel    448 Mar  9 16:16 ..
drwxr-x---@ 12 finn.carlsvi  wheel    384 Mar  9 16:16 .git
-rw-r-----@  1 finn.carlsvi  wheel    337 Mar  9 16:16 .gitignore
-rw-r-----@  1 finn.carlsvi  wheel  35147 Mar  9 16:16 LICENSE
-rw-r-----@  1 finn.carlsvi  wheel    186 Mar  9 16:16 README.md
drwxr-x---@ 13 finn.carlsvi  wheel    416 Mar  9 16:16 doc
drwxr-x---@ 77 finn.carlsvi  wheel   2464 Mar  9 16:16 dxf
drwxr-x---@  3 finn.carlsvi  wheel     96 Mar  9 16:16 links
== jscut ==
origin	https://github.com/tbfleming/jscut.git (fetch)
origin	https://github.com/tbfleming/jscut.git (push)
total 832
drwxr-x---@ 30 finn.carlsvi  wheel     960 Mar  9 16:16 .
drwxr-x---@ 14 finn.carlsvi  wheel     448 Mar  9 16:16 ..
drwxr-x---@ 13 finn.carlsvi  wheel     416 Mar  9 16:16 .git

thinking
**Implementing parallel repo scanning**
codex
I now have local checkouts for all projects, including the canonical `jscut` and `dxf2gcode` repos. I’m extracting direct evidence for profile/tabs/pockets/post support from source and docs so each rating is traceable.
exec
/bin/zsh -lc 'set -e
base=/tmp/cam_research
for d in FreeCAD/blendercam pycam opencamlib DerpCAM grid-apps jscut OpenBuilds-CAM CAMotics heekscnc dxf2gcode PathCAM; do :; done
# explicit map
repos=(
  /tmp/cam_research/FreeCAD
  /tmp/cam_research/blendercam
  /tmp/cam_research/pycam
  /tmp/cam_research/opencamlib
  /tmp/cam_research/DerpCAM
  /tmp/cam_research/grid-apps
  /tmp/cam_research/jscut
  /tmp/cam_research/OpenBuilds-CAM
  /tmp/cam_research/CAMotics
  /tmp/cam_research/heekscnc
  /tmp/cam_research/dxf2gcode
  /tmp/cam_research/PathCAM
)
for r in "${repos[@]}"; do
  echo "=== $(basename "$r") ==="
  cd "$r"
  rg -n -i "profile|contour|inside|outside|climb|conventional|pocket|trocho|adaptive|tab|bridge|holding tag|post.?processor|gcode|api|library|sdk" README* docs* src* cam* heekscnc* dxf* 2>/dev/null | head -n 40 || true
  echo
done' in /Users/finn.carlsvi/thetta succeeded in 168ms:

=== jscut ===
zsh:22: no matches found: docs*

=== OpenBuilds-CAM ===
zsh:22: no matches found: docs*

=== CAMotics ===
zsh:22: no matches found: docs*

=== heekscnc ===
zsh:22: no matches found: docs*

=== dxf2gcode ===
zsh:22: no matches found: docs*

=== PathCAM ===
zsh:22: no matches found: docs*


thinking
**Choosing ripgrep for pattern search**
exec
/bin/zsh -lc 'set -e
repos=(FreeCAD blendercam pycam opencamlib DerpCAM grid-apps jscut OpenBuilds-CAM CAMotics heekscnc dxf2gcode PathCAM)
for d in "${repos[@]}"; do
  echo "=== $d ==="
  cd "/tmp/cam_research/$d"
  rg -n -i "profile|contour|inside|outside|climb|conventional|pocket|trocho|adaptive|tab|bridge|holding tag|post.?processor|gcode|api|library|sdk" . | head -n 60 || true
  echo
done' in /Users/finn.carlsvi/thetta succeeded in 225ms:
./src/DerpCAM/gui/inventory.py:178:    CONVENTIONAL = 0
./src/DerpCAM/gui/inventory.py:179:    CLIMB = 1
./src/DerpCAM/gui/inventory.py:181:        (CONVENTIONAL, "Conventional", False),
./src/DerpCAM/gui/inventory.py:182:        (CLIMB, "Climb", True),
./src/DerpCAM/gui/inventory.py:185:class PocketStrategy(EnumClass):
./src/DerpCAM/gui/inventory.py:186:    CONTOUR_PARALLEL = 1
./src/DerpCAM/gui/inventory.py:192:        (CONTOUR_PARALLEL, "Contour-parallel"),
./src/DerpCAM/gui/inventory.py:219:    properties = [ 'rpm', 'hfeed', 'vfeed', 'maxdoc', 'offset', 'stepover', 'direction', 'extra_width', 'trc_rate', 'pocket_strategy', 'axis_angle', 'eh_diameter', 'entry_mode', 'roughing_offset', IdRefProperty('toolbit') ]
./src/DerpCAM/gui/inventory.py:221:    def new(klass, id, name, toolbit, rpm, hfeed, vfeed, maxdoc, offset, stepover, direction, extra_width, trc_rate, pocket_strategy, axis_angle, eh_diameter, entry_mode, roughing_offset):
./src/DerpCAM/gui/inventory.py:233:        res.pocket_strategy = pocket_strategy
./src/DerpCAM/gui/inventory.py:266:    def addPreset(self, id, name, rpm, hfeed, vfeed, maxdoc, offset, stepover, direction, extra_width, trc_rate, pocket_strategy, axis_angle, eh_diameter, entry_mode, roughing_offset):
./src/DerpCAM/gui/inventory.py:267:        self.presets.append(EndMillPreset.new(id, name, self, rpm, hfeed, vfeed, maxdoc, offset, stepover, direction, extra_width, trc_rate, pocket_strategy, axis_angle, eh_diameter, entry_mode, roughing_offset))
./src/DerpCAM/gui/inventory.py:354:                .addPreset(100, "Wood-roughing", 24000, 3200, 1500, 2, 0, 0.6, MillDirection.CONVENTIONAL, 0, 0, PocketStrategy.CONTOUR_PARALLEL, 0, 0.5, EntryMode.PREFER_RAMP, 0.1)
./src/DerpCAM/gui/inventory.py:355:                .addPreset(101, "Wood-finishing", 24000, 1600, 1500, 1, 0, 0.6, MillDirection.CLIMB, 0, 0, PocketStrategy.CONTOUR_PARALLEL, 0, 0.5, EntryMode.PREFER_RAMP, 0.1),
./src/DerpCAM/gui/inventory.py:357:                .addPreset(102, "Wood-roughing", 24000, 3200, 1500, 2, 0, 0.6, MillDirection.CONVENTIONAL, 0, 0, PocketStrategy.CONTOUR_PARALLEL, 0, 0.5, EntryMode.PREFER_RAMP, 0.1)
./src/DerpCAM/gui/inventory.py:358:                .addPreset(103, "Wood-finishing", 24000, 1600, 1500, 1, 0, 0.6, MillDirection.CLIMB, 0, 0, PocketStrategy.CONTOUR_PARALLEL, 0, 0.5, EntryMode.PREFER_RAMP, 0.1),
./src/DerpCAM/gui/inventory.py:360:                .addPreset(104, "Alu-risky", 16000, 500, 100, 0.5, 0, 0.4, MillDirection.CONVENTIONAL, 0, 0, PocketStrategy.CONTOUR_PARALLEL, 0, 0.5, EntryMode.PREFER_HELIX, 0.15),
./src/DerpCAM/gui/canvas.py:25:                self.document.forEachOperation(lambda item: self.renderRapids(item.renderer, owner) if item.renderer else None)
./src/DerpCAM/gui/canvas.py:28:                    owner.addRapidLine(pen, self.lastpt, PathPoint(0, 0))
./src/DerpCAM/gui/canvas.py:29:    def renderRapids(self, renderer, owner):

thinking
**Planning targeted repo feature scans**
exec
/bin/zsh -lc "cd /tmp/cam_research/blendercam && rg -n -i \"profile|pocket|tab|bridge|inside|outside|climb|conventional|post-processor|postprocessor|gcode|api\" README.md docs -g '*.md' | head -n 120" in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc "cd /tmp/cam_research/DerpCAM && rg -n -i \"profile|outside|inside|tabs|pocket|strategy|trocho|climb|conventional|g-code|dialect|api|library\" README.md src -g '*.md' | head -n 140" in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc "cd /tmp/cam_research/opencamlib && rg -n -i \"library|python|node|wasm|api|pocket|contour|tab|dropcutter|waterline|adaptive|toolpath\" README.md src/manual.hpp docs -g '*.*' | head -n 160" in /Users/finn.carlsvi/thetta succeeded in 51ms:
docs/tools.md:24:*In older versions of **Blender**, select the **BlenderCAM** tab to view the **Curve CAM Tools** panel.*
docs/tools.md:45:Makes a new curve that is cuttable from both the inside and outside. 
docs/tools.md:72:Add overcuts for slots/inserts/tabs and creates a new curve. Works on text and curve objects.  Three types of overcuts can be used: T-bone, Dog-bone, and opposite edge.
docs/tools.md:81:  - **Opposite Edge** - when placing an overcut switch to the opposite edge from the last inside corner
docs/tools.md:82:- **Max Inside Angle** - if the angle of the corner is less than max inside angle then the corner is considered an inside corner and an overcut will be placed. 
docs/tools.md:84:- **Invert** - make inside cut an outside cut or vice versa 
docs/tools.md:131:## Get Pocket Surfaces
docs/tools.md:132:This operator will detect pockets in a mesh and extract them as curves.  The curves can then be used for pocket operations.  Curves that are at the same z level will be in the same object.  All generated curve objects will be placed into the same group.
docs/tools.md:134:![Get Pocket Surface](_static/get_pocket2.png)
docs/tools.md:135:![Get Pocket Surface2](_static/get_pocket.png)
docs/tools.md:139:  * **z limit** - flat surfaces that are below the z limit are considered a pocket.
docs/tools.md:143:Operator option **z limit** controls which horizontal surfaces are considered a pocket.  If the horizontal surface z location is below the z limit then the surface is considered a pocket. Setting z limit to 0.0 (default setting) should work for most situations if mesh top is at z=0.0.
README.md:7:### [About](#-about) • [Guide](#-user-guide) • [Features](#-features) • [Post-Processors](#-post-processors) • [Files](#-files-organisation) • [Contribute](#-contribute) • [License](#-license) • [Disclaimer](#-disclaimer)
README.md:20:[**FabexCNC**](https://blendercam.com/) is an open source solution for CAM (*Computer Aided Machining*) toolpath generation and Gcode export.
README.md:39:> *You can also view the full [User, Developer and API Documentation](https://spectralvectors.github.io/blendercam/index.html)*
README.md:45:| **2D & 3D Milling Strategies** | _Profile, Pocket, Drill, Parallel, Cross, Block, Spiral, Medial Axis and more_ | ✅ |
README.md:58:| **Automatic Bridges** | _One click to add Bridges/Tabs to keep your work in place during Cutout Operations_ | ✅ |
README.md:59:| **Chain Operations** | _Combine multiple CAM operations into a Chain, Export as Gcode, or create a Chain Simulation_ | ✅ |
README.md:71:## 💻 Post-processors
README.md:91:docs/ - # User, Developer and API docs
 succeeded in 51ms:
README.md:23:* outside/inside slot milling with optional tabs, wide slots and trochoidal paths
README.md:25:* pocket milling with island support, multiple strategies available including a HSM strategy
README.md:31:* tabs placement either automatic or manual
README.md:39:* global tool/preset library
README.md:43:* G-Code output using Grbl or LinuxCNC dialects
README.md:50:* external (side) milling - producing a target shape by milling from the outside edges (HSM mode works best)
README.md:52:* rest machining - refining a coarsely-milled pocket or outline using a finer tool
README.md:60:* Python API for generating G-Code from data instead of a CAD drawing
README.md:62:* wall profile support for efficient milling of draft angles or fillets with flat end endmills, Python API only
README.md:101:support G-Code dialects used by the common commercial machine vendors. It does
 succeeded in 51ms:
rg: README.md: No such file or directory (os error 2)
src/manual.hpp:7: * OpenCAMLib is a C++ library with Python bindings under LGPL license 
src/manual.hpp:8: * for creating toolpaths for cnc machines.
src/manual.hpp:58: * <kbd>shell> apt-get install debhelper libboost-python-dev python-dev python-support </kbd>
src/manual.hpp:67: * <kbd>shell> dpkg -i ../python-opencamlib_*.deb</kbd>
src/manual.hpp:83: * Here text on how to use the library.
src/manual.hpp:270: * An algorithm for 'slicing' an STL surface at a defined Z-coordinate. Produces waterline type paths. 
src/manual.hpp:271: * These contours can be used as a starting point for area-clearing algorithms in roughing operations.
src/manual.hpp:289: *      - DropCutter for flat, filleted and ball-nose cutters. Edge test with iterative solution. 
src/manual.hpp:291: *      - DropCutter for a general APT-style cutter. This includes cutters with pointed ends and tapered cutters. 
src/manual.hpp:294: *      - presents a Push-cutter idea for calculating z-level (waterline) paths. 
docs/api.rst:1:API Documentation
docs/api.rst:50:.. doxygenclass:: ocl::KDNode
docs/adaptivepathdropcutter.rst:1:AdaptivePathDropCutter
docs/adaptivepathdropcutter.rst:4:.. doxygenclass:: ocl::AdaptivePathDropCutter
docs/conf.py:293:# If true, do not generate a @detailmenu in the "Top" node's menu.
docs/index.rst:7:   api
docs/index.rst:10:Indices and tables
docs/operations.rst:7:.. doxygenclass:: ocl::AdaptiveWaterline
docs/operations.rst:10:.. doxygenclass:: ocl::Waterline

thinking
**Querying pycam specifics**
exec
/bin/zsh -lc "cd /tmp/cam_research/pycam && rg -n -i \"profile|contour|pocket|tab|bridge|inside|outside|climb|conventional|trocho|adaptive|post|gcode|plugin|api|library\" README* INSTALL.md technical_details.txt yaml_flow_example.yml pycam -g '*.py' | head -n 180" in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc "cd /tmp/cam_research/jscut && rg -n -i \"pocket|profile|outline|inside|outside|tab|tabs|climb|conventional|gcode|post|api|library\" README* api* js* -g '*.js' -g '*.md' | head -n 180" in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc "cd /tmp/cam_research/grid-apps && rg -n -i \"pocket|profile|contour|tab|tabs|bridge|inside|outside|conventional|climb|cam|gcode|post|api|worker|plugin\" README* src web app js -g '*.js' -g '*.md' | head -n 180" in /Users/finn.carlsvi/thetta succeeded in 51ms:
yaml_flow_example.yml:66:                        type: support_bridges
yaml_flow_example.yml:85:                        type: support_bridges
yaml_flow_example.yml:201:                gcode:
yaml_flow_example.yml:220:                        type: gcode
INSTALL.md:74:go to the “Security” tab and make sure you’ve got “Allow connections from network
technical_details.txt:32:The ContourCutter extracts contours from the cutter scanlines,
README.md:5:PyCAM generates toolpaths (GCode) based on 2D or 3D models for 3-axis CNC machining.
README.md:40:* Paul: GCode stepping precision
pycam/Flow/history.py:59:        """ postpone storing individual revisions until the end of the context
pycam/Flow/parser.py:3:import pycam.Exporters.GCode.LinuxCNC
pycam/Flow/parser.py:5:import pycam.Plugins
pycam/Geometry/PointKdtree.py:13:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
pycam/Cutters/CylindricalCutter.py:14:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
pycam/PathProcessors/ContourCutter.py:13:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
pycam/PathProcessors/ContourCutter.py:26:class ContourCutter(pycam.PathProcessors.BasePathProcessor):
pycam/PathProcessors/ContourCutter.py:45:            self.polygon_extractor = PolygonExtractor(PolygonExtractor.CONTOUR)
pycam/Test/test_tools.py:13:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
pycam/Geometry/Plane.py:13:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
pycam/PathProcessors/PolygonCutter.py:14:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
pycam/Geometry/Path.py:14:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 succeeded in 51ms:
jscut.css:25:.table > tbody > tr > td {
jscut.css:30:.table > tbody > tr > td > input[type="number"], .number {
jscut.css:44:.tabsGeometry {
jscut.css:70:    outline: none;
jscut.html:13:MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
jscut.html:103:    <!-- Save GCODE Modal -->
jscut.html:104:    <div class="modal fade" id="save-gcode-modal" role="dialog" aria-labelledby="myModalLabel" aria-hidden="true">
jscut.html:110:                        Save GCODE
jscut.html:114:                    <input id="FileGetGcode1" data-bind="value: gcodeFilename">
jscut.html:118:                    <button type="button" class="btn btn-primary" onclick="saveGcodeLocalFile(function () { $('#save-gcode-modal').modal('hide') })">Local file</button>
jscut.html:119:                    <button type="button" class="btn btn-primary" id="saveGcodeGoogle1" data-bind="visible: enableGoogleDrive" onclick="saveGcodeGoogle(function () { $('#save-gcode-modal').modal('hide') })">Google Drive</button>
jscut.html:120:                    <button type="button" class="btn btn-primary" onclick="chiliSaveGcode()">ChiliPeppr</button>
jscut.html:133:                        Save GCODE
jscut.html:137:                    gcode saved to chilipeppr.com.
jscut.html:161:                    <table class="table">
jscut.html:165:                    </table>
jscut.html:212:                            <table class="table">
jscut.html:216:                            </table>
jscut.html:221:                        <div style="display:table">
jscut.html:223:                                <div style="display:table-row; background-color: #f5f5f5;">
 succeeded in 51ms:
zsh:1: no matches found: README*

thinking
**Simplifying grid-app output command**
exec
/bin/zsh -lc 'cd /tmp/cam_research/grid-apps && rg -n -i "pocket|profile|contour|tab|tabs|bridge|inside|outside|climb|conventional|gcode|post|api|worker|kiri" . | head -n 220' in /Users/finn.carlsvi/thetta succeeded in 52ms:
./src/dock/compose.yml:4:  kirimoto:
./src/main/mesh.js:8:import { api } from '../mesh/api.js';
./src/main/mesh.js:28:    return api.log.emit(...arguments);
./src/main/mesh.js:40:        db = api.db = {
./src/main/mesh.js:45:    // initialize the API (to avoid circular dependencies)
./src/main/mesh.js:46:    api.init();
./src/main/mesh.js:81:    // reload stored space when worker is ready
./src/main/mesh.js:84:    // start worker
./src/main/mesh.js:101:    self.mesh = { api };
./src/main/mesh.js:107:    const db_admin = api.db.admin;
./src/main/mesh.js:108:    const db_space = api.db.space;
./src/main/mesh.js:142:                    api.group.new(models, id).applyMeta(mcache[id])
./src/main/mesh.js:149:                api.add.sketch({ id, ...data });
./src/main/mesh.js:169:        return api.prefs.load().then(() => {
./src/main/mesh.js:170:            let { map } = api.prefs;
./src/main/mesh.js:172:            api.grid(space.grid);
./src/main/mesh.js:175:            let smodel = api.model.list().filter(m => selist.contains(m.id));
./src/main/mesh.js:176:            let sgroup = api.group.list().filter(m => selist.contains(m.id));
./src/main/mesh.js:178:            let tmodel = api.model.list().filter(m => tolist.contains(m.id));
./src/main/mesh.js:179:            let tgroup = api.group.list().filter(m => tolist.contains(m.id));

thinking
**Listing CNC tool keywords**
codex
I’ve confirmed core feature signals for BlenderCAM, DerpCAM, PyCAM, jscut, and Kiri:Moto. I’m now finishing the remaining six projects and then I’ll score integration fit specifically for Rust embedding/extraction.
exec
/bin/zsh -lc "cd /tmp/cam_research/CAMotics && rg -n -i \"simulate|simulation|gcode|toolpath generation|cam|post|pocket|profile|tab|tabs|operation\" README* src docs -g '*.md' -g '*.cpp' -g '*.h' | head -n 220" in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc 'cd /tmp/cam_research/OpenBuilds-CAM && rg -n -i "pocket|profile|inside|outside|tab|tabs|bridge|conventional|climb|gcode|post|grbl|toolpath|operation" . | head -n 220' in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc "cd /tmp/cam_research/heekscnc && rg -n -i \"profile|pocket|inside|outside|tab|bridge|climb|conventional|post|gcode|python|api|script\" README* src -g '*.cpp' -g '*.h' -g '*.py' | head -n 240" in /Users/finn.carlsvi/thetta succeeded in 51ms:
rg: docs: No such file or directory (os error 2)
README.md:1:![CAMotics Logo][1]
README.md:3:CAMotics is an Open-Source software which can simulate 3-axis NC
README.md:4:machining. It is a fast, flexible and user-friendly simulation
README.md:5:software for the DIY and Open-Source community.  CAMotics works on
README.md:11:machines. One of the major reasons for this is a lack of Open-Source simulation
README.md:12:and CAM (3D model to tool path conversion) software. CAM and NC machine
README.md:13:simulation present some very difficult programming problems, as is evidenced by
README.md:14:30+ years of academic papers on these topics. Whereas, 3D printing simulation
README.md:18:Being able to simulate is a critical part of creating CNC tool
README.md:20:measuring; it's both dangerous and expensive. With CAMotics you can
README.md:21:preview the results of your cutting operation before you fire up your
README.md:26:See http://camotics.org/
README.md:32:at https://buildbotics.com/.  Buildbotics purchases help support CAMotics.
README.md:39:[CAMotics Website](http://camotics.org/download.html).
README.md:42:This section describes how to build CAMotics from source on Debian based
README.md:45:at http://camotics.org/download.html  There are also prebuilt Debian packages
README.md:64:environment variable CBANG_HOME so the CAMotics build system can find it
README.md:71:## Building CAMotics
README.md:72:Clone the CAMotics git repository and build the software using scons:
 succeeded in 50ms:
Total output lines: 220

./index.html:46:			/* Navbar links styled as tabs */
./index.html:127:								<ul class="tabs-holder">
./index.html:150:												<button class="ribbon-icon-button" onclick="storeUndo(true); objectsInScene.length = 0; toolpathsInScene.length = 0; trashGcode(); fillTree(); $('.workspaceTitle').html('');">
./index.html:416:									<input type="checkbox" data-role="checkbox" data-caption="Toolpaths" checked data-style="2" id="view-toolpath">
./index.html:417:									<input type="checkbox" data-role="checkbox" data-caption="GCODE" checked data-style="2" id="view-gcode">
./index.html:469:								<span class="title">2. Configure Toolpaths</span>
./index.html:476:									<button class="button" id="addJobBtn" onclick="addJob(-1);"><span class="icon fa-fw fa fa-plus"></span><span class="addJobBtn-text">Please select Vector(s) to create toolpaths</span></button>
./index.html:478:									<ul class="d-menu" data-role="dropdown" id="toolpathsmenu">
./index.html:486:								<div class="pt-2" id="toolpathactivity" style="display: none;">
./index.html:490:										<h6>Calculating Toolpaths...</h6>
./index.html:492:								<div id="toolpathtree">
./index.html:506:								<button class="button drop-shadow" style="width: 300px;" title=" Generate G-Code" id="generatetpgcode" onclick="makeGcode();"><span class="fa fa-cubes fa-fw icon"></span>Generate G-Code</button>
./index.html:521:								<button class="button drop-shadow" title="Save G-Code" id="gcodesavebtn2" onclick="saveFile();"><span class="fa fa-save fa-fw icon"></span>Save G-Code</button>
./index.html:522:								<button class="button drop-shadow" title="Preview G-Code" id="gcodeviewbtn2" onclick="previewFile();"><span id="gcodepreviewicon" class="fa fa-eye fa-fw icon fg-gray"></span></button>
./index.html:523:								<button class="button drop-shadow" title="Clear G-Code" id="gcodetrashbtn2" onclick="trashGcode();"><span id="trashicon" class="fa fa-trash fa-fw fg-gray icon"></span></button>
./index.html:593:		METRO_HOTKEYS_FILTER_CONTENT_EDITABLE = false;
./index.html:649:	<script type="text/javascript" src="js/advanced-cam-gcode.js"></script>
./index.html:683:	<!-- Gcode Viewer -->
 succeeded in 51ms:
README.md:55:If you choose a user-writable prefix, superuser privileges are not needed:
README.md:62:HeeksCNC requires additional python modules at runtime:
README.md:63: - python module built by libarea
README.md:64: - python module built by opencamlib
src/SketchOp.h:38:	Python AppendTextToProgram();
src/ScriptOpDlg.cpp:1:// ScriptOpDlg.cpp
src/ScriptOpDlg.cpp:6:#include "ScriptOpDlg.h"
src/ScriptOpDlg.cpp:9:#include "ScriptOp.h"
src/ScriptOpDlg.cpp:15:BEGIN_EVENT_TABLE(ScriptOpDlg, OpDlg)
src/ScriptOpDlg.cpp:16:    EVT_TEXT(ID_SCRIPT_TXT, ScriptOpDlg::OnScriptText)
src/ScriptOpDlg.cpp:17:    EVT_BUTTON(wxID_HELP, ScriptOpDlg::OnHelp)
src/ScriptOpDlg.cpp:18:END_EVENT_TABLE()
src/ScriptOpDlg.cpp:20:ScriptOpDlg::ScriptOpDlg(wxWindow *parent, CScriptOp* object, const wxString& title, bool top_level)
src/ScriptOpDlg.cpp:25:		wxStaticText *static_label = new wxStaticText(this, wxID_ANY, _("Script"));
src/ScriptOpDlg.cpp:27:		sizer_vertical->Add( m_txtScript = new wxTextCtrl(this, ID_SCRIPT_TXT, wxEmptyString, wxDefaultPosition, wxSize(300, 200), wxTE_MULTILINE | wxTE_DONTWRAP | wxTE_RICH | wxTE_RICH2), 1, wxLEFT | wxALIGN_RIGHT | wxALIGN_CENTER_VERTICAL, control_border );
src/ScriptOpDlg.cpp:34:		m_txtScript->SetFocus();
src/ScriptOpDlg.cpp:38:void ScriptOpDlg::GetDataRaw(HeeksObj* object)
src/ScriptOpDlg.cpp:40:	((CScriptOp*)object)->m_str = m_txtScript->GetValue();
src/ScriptOpDlg.cpp:45:void ScriptOpDlg::SetFromDataRaw(HeeksObj* object)
src/ScriptOpDlg.cpp:47:	m_txtScript->SetValue(((CScriptOp*)object)->m_str);

thinking
**Planning targeted queries for post processors**
exec
/bin/zsh -lc "cd /tmp/cam_research/PathCAM && rg -n -i \"inside|outside|profile|pocket|tab|tabs|bridge|gcode|router|pathplanner|cut|offset|climb|conventional|library|api\" README.md Router PathCAM -g '*.cs' -g '*.md' | head -n 260" in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc 'cd /tmp/cam_research/FreeCAD && rg -n -i "Side|Inside|Outside|Direction|CW|CCW|Tags|generateTags|Width|Height|Angle|Spacing|Pocket|Adaptive|Post/scripts|grbl|linuxcnc|fanuc|mach3|masso" src/Mod/CAM/Path/Op/Profile.py src/Mod/CAM/Path/Dressup/Tags.py src/Mod/CAM/Path/Dressup/Gui/Tags.py src/Mod/CAM/Path/Op/Pocket.py src/Mod/CAM/Path/Op/Adaptive.py src/Mod/CAM/Path/Post/scripts | head -n 260' in /Users/finn.carlsvi/thettaexec
/bin/zsh -lc "cd /tmp/cam_research/dxf2gcode && rg -n -i \"inside|outside|contour|profile|pocket|tab|bridge|climb|conventional|gcode|post|toolpath|offset\" README.md source dxf2gcode.py -g '*.py' -g '*.md' | head -n 240" in /Users/finn.carlsvi/thetta succeeded in 51ms:
rg: PathCAM: No such file or directory (os error 2)
README.md:4:PathCAM - Toolpath generation software for CNC robots!  PathCAM is a simple, easy to use tool for generating 2.5D toolpaths to cut shapes from stock material using a CNC Router.  PathCAM can connect directly to some CNC robots, and can export simple .gcode for others.
README.md:33:Start by loading a .stl or .obj file - you can just drag & drop from the file system, or use the Open File button.  Make sure the dropdown for scale is set correctly before loading the file (most files on Thingiverse are in millimeters).  You can move the models (green things) and the tabs (orange things) around to suit your needs.  Once you've got everything where you want it, try to generate some toolpaths.
README.md:36:* Add Perimeter Paths: Adds toolpaths which follow the edges of the object.  The paths will be divided into layers depending on "Max Cut Depth", and will do two passes along each edge: one rough cut, removing the bulk of the material, and a clean cut trimming the edge to the exact dimension.
README.md:38:With the tool paths generated, you can save them to a .gcode file or run them directly from PathCAM on specific robots (Including machines running GRBL!!!).  Connections to more robots will be added in the future - if you have one in mind, say something and maybe it will be added sooner!
README.md:47:1. Enter the material height in the "Z Go" box, and the negative of that value in the Z Offset box.  Hit return while in the Z Offset box, then use the mouse scroll wheel or up/down buttons to jog the Z axis to the surface of the material.
README.md:48:1. Load an object (.STL, .DAE, raw .gcode), ensure the speeds, heights, and other settings are to your liking, then click "Run" to start sending commands to your robot.
README.md:56:* [Clipper](http://www.angusj.com/delphi/clipper.php): A great polygon clipping tool - PathCAM uses Clipper to offset toolpaths from the raw sliced object layers.
README.md:57:* [STLdotNET](https://github.com/QuantumConcepts/STLdotNET): STL loading library used by PathCAM to open .STL files.  Works Great!
Router/Properties/AssemblyInfo.cs:8:[assembly: AssemblyTitle("GCode")]
Router/Properties/AssemblyInfo.cs:12:[assembly: AssemblyProduct("GCode")]
Router/Router.cs:12: * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
Router/Router.cs:31:namespace Router
Router/Router.cs:33:    public class Router
Router/Router.cs:39:        private float move_height = 0.550f; // How high above the surface to move the router
Router/Router.cs:40:        private float max_cut_depth = .250f; // Maximum cut depth in inches
Router/Router.cs:42:        float tabHeight = 0.050f;
Router/Router.cs:45:        public Router()
Router/Router.cs:69:        public float TabHeight
Router/Router.cs:71:            get { return tabHeight; }
 succeeded in 51ms:
src/Mod/CAM/Path/Dressup/Gui/Tags.py:24:from PySide import QtCore, QtGui
src/Mod/CAM/Path/Dressup/Gui/Tags.py:25:from PySide.QtCore import QT_TRANSLATE_NOOP
src/Mod/CAM/Path/Dressup/Gui/Tags.py:31:import Path.Dressup.Tags as PathDressupTag
src/Mod/CAM/Path/Dressup/Gui/Tags.py:59:        self.form = FreeCADGui.PySideUic.loadUi(":/panels/HoldingTagsEdit.ui")
src/Mod/CAM/Path/Dressup/Gui/Tags.py:63:            FreeCAD.ActiveDocument.openTransaction("Edit HoldingTags Dress-up")
src/Mod/CAM/Path/Dressup/Gui/Tags.py:74:        self.tags = None
src/Mod/CAM/Path/Dressup/Gui/Tags.py:118:    def getTags(self, includeCurrent):
src/Mod/CAM/Path/Dressup/Gui/Tags.py:119:        tags = []
src/Mod/CAM/Path/Dressup/Gui/Tags.py:120:        index = self.form.lwTags.currentRow()
src/Mod/CAM/Path/Dressup/Gui/Tags.py:121:        for i in range(0, self.form.lwTags.count()):
src/Mod/CAM/Path/Dressup/Gui/Tags.py:122:            item = self.form.lwTags.item(i)
src/Mod/CAM/Path/Dressup/Gui/Tags.py:128:                tags.append((x, y, enabled))
src/Mod/CAM/Path/Dressup/Gui/Tags.py:129:        return tags
src/Mod/CAM/Path/Dressup/Gui/Tags.py:132:        width = FreeCAD.Units.Quantity(self.form.ifWidth.text()).Value
src/Mod/CAM/Path/Dressup/Gui/Tags.py:133:        height = FreeCAD.Units.Quantity(self.form.ifHeight.text()).Value
src/Mod/CAM/Path/Dressup/Gui/Tags.py:134:        angle = self.form.dsbAngle.value()
src/Mod/CAM/Path/Dressup/Gui/Tags.py:137:        tags = self.getTags(True)
src/Mod/CAM/Path/Dressup/Gui/Tags.py:140:        for i, (x, y, enabled) in enumerate(tags):
src/Mod/CAM/Path/Dressup/Gui/Tags.py:145:        if width != self.obj.Width:
src/Mod/CAM/Path/Dressup/Gui/Tags.py:146:            self.obj.Width = width
 succeeded in 52ms:
rg: dxf2gcode.py: No such file or directory (os error 2)
README.md:1:dxf2gcode
README.md:4:This is a fork of https://code.google.com/p/dxf2gcode/.
source/make_exe.py:11:file = "dxf2gcode"
source/make_exe.py:12:icon= "%s/DXF2GCODE-001.ico" %filepfad
source/Gui/WpZero.py:4:#dxf2gcode_b02_point
source/Gui/WpZero.py:10:#dxf2gcode is free software; you can redistribute it and/or modify
source/Gui/WpZero.py:17:#MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
source/Gui/WpZero.py:39:        self.setFlag(QtGui.QGraphicsItem.ItemIsSelectable, False)
source/Gui/myTreeView.py:97:        item, even when we only use internals move inside the treeView.
source/Gui/myTreeView.py:122:            if drop_model_index.isValid() and relative_position != QtGui.QTreeView.OnViewport: #drop position is computable from a real element
source/Gui/myTreeView.py:254:        Function called by QT when a key has been pressed inside the treeView. Subclassed in order to call a callback function
source/epydoc-gen.py:11:# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
source/Gui/Arrow.py:4:#dxf2gcode_b02_point
source/Gui/Arrow.py:10:#dxf2gcode is free software; you can redistribute it and/or modify
source/Gui/Arrow.py:17:#MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
source/Gui/Arrow.py:55:        self.setFlag(QtGui.QGraphicsItem.ItemIsSelectable, False)
source/d2gexceptions.py:10:# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
source/Gui/myCanvasClass.py:32:    This is the used Canvas to print the graphical interface of dxf2gcode. 
source/Gui/myCanvasClass.py:461:    This is the Canvas used to print the graphical interface of dxf2gcode.

thinking
**Planning robust GitHub metrics extraction**
exec
/bin/zsh -lc "python3 - <<'PY'
import json, urllib.request, re
repos=[
'FreeCAD/FreeCAD',
'vilemduha/blendercam',
'SebKuzminsky/pycam',
'aewallin/opencamlib',
'kfoltman/DerpCAM',
'GridSpace/grid-apps',
'tbfleming/jscut',
'OpenBuilds/OpenBuilds-CAM',
'CauldronDevelopmentLLC/CAMotics',
'Heeks/heekscnc',
'workflo/dxf2gcode',
'xenovacivus/PathCAM']

def get(url):
    req=urllib.request.Request(url,headers={'User-Agent':'cam-research'})
    with urllib.request.urlopen(req, timeout=30) as r:
        return r.read(), r.headers, r.status

for r in repos:
    print('===',r,'===')
    try:
        b,h,s=get(f'https://api.github.com/repos/{r}')
        j=json.loads(b)
    except Exception as e:
        print('repo_error',e)
        continue
    print('url',j.get('html_url'))
    print('language',j.get('language'),'license', (j.get('license') or {}).get('spdx_id'))
    print('stars',j.get('stargazers_count'),'forks',j.get('forks_count'),'watchers',j.get('subscribers_count'))
    print('pushed_at',j.get('pushed_at'),'updated_at',j.get('updated_at'))
    # contributors count via link last page
    contrib='unknown'
    try:
        _,h2,_=get(f'https://api.github.com/repos/{r}/contributors?per_page=1&anon=1')
        link=h2.get('Link','')
        m=re.search(r'[?&]page=(\\d+)>; rel=\"last\"',link)
        contrib=int(m.group(1)) if m else 1
    except Exception as e:
        contrib=f'err:{e}'
    print('contributors',contrib)
    # releases
    rel_count='unknown'; latest='none'; latest_date='none'
    try:
        b,_,_=get(f'https://api.github.com/repos/{r}/releases?per_page=100')
        arr=json.loads(b)
        rel_count=len(arr)
        if arr:
            latest=arr[0].get('tag_name')
            latest_date=arr[0].get('published_at')
    except Exception as e:
        rel_count=f'err:{e}'
    print('releases',rel_count,'latest_release',latest,latest_date)
PY" in /Users/finn.carlsvi/thetta succeeded in 6.15s:
=== FreeCAD/FreeCAD ===
url https://github.com/FreeCAD/FreeCAD
language C++ license LGPL-2.1
stars 28960 forks 5247 watchers 538
pushed_at 2026-03-09T21:12:35Z updated_at 2026-03-09T22:58:56Z
contributors 960
releases 84 latest_release weekly-2026.03.04 2026-03-04T00:44:59Z
=== vilemduha/blendercam ===
url https://github.com/vilemduha/blendercam
language G-code license GPL-3.0
stars 553 forks 132 watchers 49
pushed_at 2026-01-01T00:44:47Z updated_at 2026-03-03T22:07:50Z
contributors 51
releases 8 latest_release 3.0.2 2026-01-01T00:43:57Z
=== SebKuzminsky/pycam ===
repo_error HTTP Error 403: rate limit exceeded
=== aewallin/opencamlib ===
repo_error HTTP Error 403: rate limit exceeded
=== kfoltman/DerpCAM ===
repo_error HTTP Error 403: rate limit exceeded

thinking
**Designing fallback metrics collection**
exec
/bin/zsh -lc "python3 - <<'PY'
import os,subprocess,json
base='/tmp/cam_research'
repos=[
('FreeCAD','FreeCAD/FreeCAD',28960,5247),
('blendercam','vilemduha/blendercam',553,132),
('pycam','SebKuzminsky/pycam',391,123),
('opencamlib','aewallin/opencamlib',496,164),
('DerpCAM','kfoltman/DerpCAM',34,6),
('grid-apps','GridSpace/grid-apps',830,184),
('jscut','tbfleming/jscut',327,None),
('OpenBuilds-CAM','OpenBuilds/OpenBuilds-CAM',340,800),
('CAMotics','CauldronDevelopmentLLC/CAMotics',708,159),
('heekscnc','Heeks/heekscnc',103,47),
('dxf2gcode','workflo/dxf2gcode',45,None),
('PathCAM','xenovacivus/PathCAM',238,63),
]
for d,repo,stars,forks in repos:
    p=os.path.join(base,d)
    def run(cmd):
        return subprocess.check_output(cmd,shell=True,cwd=p,text=True).strip()
    try:
        last=run(\"git log -1 --format='%cI'\")
    except Exception:
        last='unknown'
    try:
        contrib=int(run(\"git shortlog -sn --all | wc -l\"))
    except Exception:
        contrib=-1
    try:
        tags=int(run(\"git tag | wc -l\"))
        latest_tag=run(\"git describe --tags --abbrev=0 2>/dev/null || echo none\")
    except Exception:
        tags=-1;latest_tag='unknown'
    print(json.dumps({
      'repo':repo,'dir':d,'last_commit':last,'contributors_git':contrib,
      'tags':tags,'latest_tag':latest_tag,'stars_snapshot':stars,'forks_snapshot':forks
    }))
PY" in /Users/finn.carlsvi/thetta succeeded in 2.77s:
{"repo": "FreeCAD/FreeCAD", "dir": "FreeCAD", "last_commit": "2026-03-09T22:12:34+01:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 28960, "forks_snapshot": 5247}
{"repo": "vilemduha/blendercam", "dir": "blendercam", "last_commit": "2026-01-01T00:43:56Z", "contributors_git": 1, "tags": 1, "latest_tag": "3.0.2", "stars_snapshot": 553, "forks_snapshot": 132}
{"repo": "SebKuzminsky/pycam", "dir": "pycam", "last_commit": "2021-11-25T08:33:45-07:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 391, "forks_snapshot": 123}
{"repo": "aewallin/opencamlib", "dir": "opencamlib", "last_commit": "2025-02-12T10:23:45+02:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 496, "forks_snapshot": 164}
{"repo": "kfoltman/DerpCAM", "dir": "DerpCAM", "last_commit": "2023-03-23T22:07:22Z", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 34, "forks_snapshot": 6}
{"repo": "GridSpace/grid-apps", "dir": "grid-apps", "last_commit": "2026-01-11T15:29:15-05:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 830, "forks_snapshot": 184}
{"repo": "tbfleming/jscut", "dir": "jscut", "last_commit": "2023-02-02T09:26:28-05:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 327, "forks_snapshot": null}
{"repo": "OpenBuilds/OpenBuilds-CAM", "dir": "OpenBuilds-CAM", "last_commit": "2025-03-04T20:38:33+02:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 340, "forks_snapshot": 800}
{"repo": "CauldronDevelopmentLLC/CAMotics", "dir": "CAMotics", "last_commit": "2026-02-10T14:49:26+02:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 708, "forks_snapshot": 159}
{"repo": "Heeks/heekscnc", "dir": "heekscnc", "last_commit": "2021-10-14T09:25:37+01:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 103, "forks_snapshot": 47}
{"repo": "workflo/dxf2gcode", "dir": "dxf2gcode", "last_commit": "2015-04-29T12:54:22+02:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 45, "forks_snapshot": null}
{"repo": "xenovacivus/PathCAM", "dir": "PathCAM", "last_commit": "2023-04-16T17:52:15-07:00", "contributors_git": 1, "tags": 0, "latest_tag": "none", "stars_snapshot": 238, "forks_snapshot": 63}

thinking
**Testing alternative contributors source**
exec
/bin/zsh -lc 'for r in tbfleming/jscut workflo/dxf2gcode SebKuzminsky/pycam; do echo "== $r =="; curl -sL "https://github.com/$r/graphs/contributors-data" | head -c 200; echo; echo; done' in /Users/finn.carlsvi/thetta succeeded in 1.30s:


== workflo/dxf2gcode ==


== SebKuzminsky/pycam ==



thinking
**Evaluating contributor count approach**
exec
/bin/zsh -lc 'set -e
for d in jscut dxf2gcode pycam DerpCAM PathCAM heekscnc; do
  cd /tmp/cam_research/$d
  git fetch --unshallow >/dev/null 2>&1 || true
  echo "== $d =="
  echo -n "contributors_git="; git shortlog -sn --all | wc -l
  echo -n "last_commit="; git log -1 --format='"'%cI'
  echo -n \"tag_count=\"; git tag | wc -l
  echo -n \"latest_tag=\"; git describe --tags --abbrev=0 2>/dev/null || echo none
  echo
done" in /Users/finn.carlsvi/thetta succeeded in 9.29s:
== jscut ==
contributors_git=       4
last_commit=2023-02-02T09:26:28-05:00
tag_count=       0
latest_tag=none

== dxf2gcode ==
contributors_git=       2
last_commit=2015-04-29T12:54:22+02:00
tag_count=       0
latest_tag=none

== pycam ==
contributors_git=      23
last_commit=2021-11-25T08:33:45-07:00
tag_count=       5
latest_tag=v0.6.2

== DerpCAM ==
contributors_git=       1

thinking
**Evaluating contributor data and releases**
exec
/bin/zsh -lc "cd /tmp/cam_research/OpenBuilds-CAM && rg -n \"CNC:|camOperation|Pocket|Vector \\(path inside\\)|Vector \\(path outside\\)|tab|Climb|Conventional|grbl|post\" workers/toolpath/worker/toolpathworker.js js -g '*.js' | head -n 260" in /Users/finn.carlsvi/thetta succeeded in 52ms:
js/advanced-cam-gcode.js:51:          if (toolpathsInScene[j].userData.camOperation == undefined) {
js/advanced-cam-gcode.js:59:            if (toolpathsInScene[j].userData.camOperation.indexOf('Drill') == 0) {
js/advanced-cam-gcode.js:73:            if (toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:85:            if (toolpathsInScene[j].userData.camOperation.indexOf('Plasma') == 0) {
js/advanced-cam-gcode.js:90:            if (parseInt(Passes) && toolpathsInScene[j].userData.camOperation.indexOf('Laser') == 0) {
js/advanced-cam-gcode.js:147:  g += "; Operation " + index + ": " + toolpathsInScene[index].userData.camOperation + "\n";
js/advanced-cam-gcode.js:148:  if (toolpathsInScene[j].userData.camOperation.indexOf('CNC') == 0 || toolpathsInScene[j].userData.camOperation.indexOf('Drill') == 0) {
js/advanced-cam-gcode.js:150:  } else if (toolpathsInScene[j].userData.camOperation.indexOf('Plasma') == 0) {
js/advanced-cam-gcode.js:152:  } else if (toolpathsInScene[j].userData.camOperation.indexOf('Laser') == 0) {
js/advanced-cam-gcode.js:154:  } else if (toolpathsInScene[j].userData.camOperation.indexOf('Drag') == 0) {
js/advanced-cam-gcode.js:156:  } else if (toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:160:  if (toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:194:      if (toolpathsInScene[index].userData.camOperation.indexOf('CNC') == 0 || toolpathsInScene[index].userData.camOperation.indexOf('Drill') == 0) {
js/advanced-cam-gcode.js:200:    if (toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:233:          if (!child.userData.closed && toolpathsInScene[j].userData.camOperation.indexOf('Plasma') != 0) {
js/advanced-cam-gcode.js:314:              if (toolpathsInScene[j].userData.camOperation.indexOf('Plasma') == 0) {
js/advanced-cam-gcode.js:342:              } else if (toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:362:            if (!toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:424:            if (toolpathsInScene[j].userData.camOperation.indexOf('Pen') == 0) {
js/advanced-cam-gcode.js:443:                if (toolpathsInScene[index].userData.camOperation.indexOf('CNC') == 0 || toolpathsInScene[index].userData.camOperation.indexOf('Drill') == 0) {

thinking
**Reviewing API documentation details**
exec
/bin/zsh -lc "cd /tmp/cam_research/grid-apps && sed -n '1,220p' README.md" in /Users/finn.carlsvi/thetta succeeded in 51ms:
# Grid.Space Applications

![GitHub commit activity](https://img.shields.io/github/commit-activity/w/GridSpace/grid-apps)
![GitHub package.json version (branch)](https://img.shields.io/github/package-json/v/GridSpace/grid-apps/rel-3.9)
![GitHub package.json version (branch)](https://img.shields.io/github/package-json/v/GridSpace/grid-apps/rel-4.0)
![GitHub package.json version (branch)](https://img.shields.io/github/package-json/v/GridSpace/grid-apps/rel-4.1)

![GitHub contributors](https://img.shields.io/github/contributors/GridSpace/grid-apps)
![GitHub last commit](https://img.shields.io/github/last-commit/GridSpace/grid-apps)
![GitHub](https://img.shields.io/github/license/GridSpace/grid-apps)

# Community

[Discord](https://discord.com/invite/suyCCgr) -- Live Chat
[Forums](https://forum.grid.space/) -- Long Form and Archival Discussion
[YouTube](https://www.youtube.com/c/gridspace) -- Content when I have time

# Free and Open Source

Kiri:Moto and Mesh:Tool are completely open source and free for use without restriction. Over 13 years in development, this passion project has grown well beyond its original scope. It has consumed most of my free time for many years. Please consider donating to support continued development GitHub sponsorship or PayPal.

thinking
**Checking license type**
exec
/bin/zsh -lc "cd /tmp/cam_research/CAMotics && ls | rg -i 'license|copying' && head -n 20 COPYING" in /Users/finn.carlsvi/thetta succeeded in 52ms:
COPYING
LICENSE
                    GNU GENERAL PUBLIC LICENSE
                       Version 2, June 1991

 Copyright (C) 1989, 1991 Free Software Foundation, Inc.,
 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The licenses for most software are designed to take away your
freedom to share and change it.  By contrast, the GNU General Public
License is intended to guarantee your freedom to share and change free
software--to make sure the software is free for all its users.  This
General Public License applies to most of the Free Software
Foundation's software and to any other program whose authors commit to
using it.  (Some other Free Software Foundation software is covered by
the GNU Lesser General Public License instead.)  You can apply it to

thinking
**Verifying license and README paths**
exec
/bin/zsh -lc 'cd /tmp/cam_research/opencamlib && ls -1 | head -n 40' in /Users/finn.carlsvi/thetta succeeded in 53ms:
CMakeLists.txt
COPYING
MANIFEST.in
README.rst
cmake
docs
examples
install.sh
pyproject.toml
setup.cfg
setup.py
src
stl

thinking
**Listing CAM postprocessor files**
exec
/bin/zsh -lc 'cd /tmp/cam_research/pycam && find pycam/Exporters/GCode -maxdepth 2 -type f -print' in /Users/finn.carlsvi/thetta succeeded in 52ms:
pycam/Exporters/GCode/LinuxCNC.py
pycam/Exporters/GCode/__init__.py

thinking
**Listing Blendercam post processors**
exec
/bin/zsh -lc 'cd /tmp/cam_research/blendercam && ls -1 scripts/addons/cam/post_processors 2>/dev/null || ls -1 post_processors' in /Users/finn.carlsvi/thetta exited 1 in 53ms:
ls: post_processors: No such file or directory

thinking
**Locating post_processors folder**
exec
/bin/zsh -lc "cd /tmp/cam_research/blendercam && rg --files | rg 'post_processors|postprocessors|nc/' | head -n 80" in /Users/finn.carlsvi/thetta succeeded in 53ms:
fabex/post_processors/siegkx1_read.py
fabex/post_processors/series1.py
fabex/post_processors/cad_read.py
fabex/post_processors/nc.py
fabex/post_processors/centroid1.py
fabex/post_processors/heiden.py
fabex/post_processors/rez2_read.py
fabex/post_processors/iso_crc_read.py
fabex/post_processors/lathe1.py
fabex/post_processors/recreator.py
fabex/post_processors/anilam_crusader_m.py
fabex/post_processors/hm50.py
fabex/post_processors/iso.py
fabex/post_processors/nc_read.py
fabex/post_processors/heiden_read.py
fabex/post_processors/num_reader.py
fabex/post_processors/emc2_read.py
fabex/post_processors/gantry_router.py
fabex/post_processors/mach3_read.py
fabex/post_processors/centroid1_read.py

thinking
**Assessing CAM postprocessor and tab details**
exec
/bin/zsh -lc "cd /tmp/cam_research/jscut && ls -1 api/js && sed -n '1,220p' api/js/cam.js" in /Users/finn.carlsvi/thetta succeeded in 53ms:
cam.js
data.js
geometry.js
parseGcode.js
svg.js
// Copyright 2014 Todd Fleming
//
// This file is part of jscut.
//
// jscut is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// jscut is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License

thinking
**Assessing CAM and CNC library options**
exec
/bin/zsh -lc 'cd /tmp/cam_research/heekscnc && rg -n "class CProfile|m_cut_mode|Conventional|Climb|inside|outside|profile" src/Profile.cpp src/Profile.h | head -n 180' in /Users/finn.carlsvi/thetta succeeded in 51ms:
src/Profile.h:15:class CProfile;
src/Profile.h:18:class CProfileParams{
src/Profile.h:28:		eConventional,
src/Profile.h:29:		eClimb
src/Profile.h:31:	eCutMode m_cut_mode;
src/Profile.h:47:	bool m_end_beyond_full_profile;
src/Profile.h:67:class CProfile: public CSketchOp{
src/Profile.h:72:	CProfileParams m_profile_params;
src/Profile.cpp:43:	m_cut_mode = eConventional;
src/Profile.cpp:60:	m_end_beyond_full_profile = false;
src/Profile.cpp:66:	m_finishing_cut_mode = eConventional;
src/Profile.cpp:74:		((CProfile*)object)->m_profile_params.m_tool_on_side = CProfileParams::eLeftOrOutside;
src/Profile.cpp:77:		((CProfile*)object)->m_profile_params.m_tool_on_side = CProfileParams::eRightOrInside;
src/Profile.cpp:80:		((CProfile*)object)->m_profile_params.m_tool_on_side = CProfileParams::eOn;
src/Profile.cpp:88:	((CProfile*)object)->m_profile_params.m_cut_mode = (CProfileParams::eCutMode)value;
src/Profile.cpp:92:static void on_set_auto_roll_on(bool value, HeeksObj* object){((CProfile*)object)->m_profile_params.m_auto_roll_on = value; heeksCAD->RefreshProperties();}
src/Profile.cpp:93:static void on_set_roll_on_point(const double* vt, HeeksObj* object){memcpy(((CProfile*)object)->m_profile_params.m_roll_on_point, vt, 3*sizeof(double));}
src/Profile.cpp:94:static void on_set_roll_radius(double value, HeeksObj* object){((CProfile*)object)->m_profile_params.m_auto_roll_radius = value; ((CProfile*)object)->WriteDefaultValues();}
src/Profile.cpp:95:static void on_set_auto_roll_off(bool value, HeeksObj* object){((CProfile*)object)->m_profile_params.m_auto_roll_off = value; heeksCAD->RefreshProperties();}
src/Profile.cpp:96:static void on_set_roll_off_point(const double* vt, HeeksObj* object){memcpy(((CProfile*)object)->m_profile_params.m_roll_off_point, vt, 3*sizeof(double));}

thinking
**Assessing repo status and licenses**
exec
/bin/zsh -lc 'cd /tmp/cam_research/pycam && head -n 25 LICENSE.TXT' in /Users/finn.carlsvi/thetta succeeded in 53ms:
  pycam - CNC Toolpath Generation in python
  =========================================

  Copyright (C) 2008 Lode Leroy
  -----------------------------

  This program is free software; you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation; either version 3 of the License, or
  (at your option) any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <https://www.gnu.org/licenses/>.

  See COPYING

thinking
**Checking project license**
exec
/bin/zsh -lc "cd /tmp/cam_research/heekscnc && ls | rg -i 'copy|license' && head -n 20 COPYING" in /Users/finn.carlsvi/thetta succeeded in 51ms:
COPYING
HeeksCAD is covered by the new BSD license

Copyright (c) 2008, Daniel Heeks
All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

    * Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
    * Neither the name of Daniel Heeks nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

thinking
**Checking license type**
exec
/bin/zsh -lc 'cd /tmp/cam_research/dxf2gcode && head -n 25 LICENSE' in /Users/finn.carlsvi/thetta succeeded in 52ms:
                    GNU GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <http://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU General Public License is a free, copyleft license for
software and other kinds of works.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to
your programs, too.

thinking
**Checking license status**
exec
/bin/zsh -lc "cd /tmp/cam_research/PathCAM && ls | rg -i 'license|copy' && head -n 25 LICENSE" in /Users/finn.carlsvi/thetta succeeded in 52ms:
LICENSE
GNU GENERAL PUBLIC LICENSE
                       Version 3, 29 June 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <http://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU General Public License is a free, copyleft license for
software and other kinds of works.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to

thinking
**Identifying AGPL license**
exec
/bin/zsh -lc "cd /tmp/cam_research/OpenBuilds-CAM && ls | rg -i 'license|copy' && head -n 10 LICENSE.md" in /Users/finn.carlsvi/thetta exited 1 in 52ms:
LICENSE
head: LICENSE.md: No such file or directory

thinking
**Checking license type**
exec
/bin/zsh -lc 'cd /tmp/cam_research/OpenBuilds-CAM && head -n 18 LICENSE' in /Users/finn.carlsvi/thetta succeeded in 53ms:
                    GNU AFFERO GENERAL PUBLIC LICENSE
                       Version 3, 19 November 2007

 Copyright (C) 2007 Free Software Foundation, Inc. <https://fsf.org/>
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

                            Preamble

  The GNU Affero General Public License is a free, copyleft license for
software and other kinds of works, specifically designed to ensure
cooperation with the community in the case of network server software.

  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
our General Public Licenses are intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.

thinking
**Compiling CAM project metrics**
exec
/bin/zsh -lc "cd /tmp/cam_research/heekscnc && rg -n \"post|PostProcess|linuxcnc|mach3|grbl|fanuc|centroid|emc\" src pycnc nc -g '*.py' -g '*.cpp' -g '*.h' | head -n 140" in /Users/finn.carlsvi/thetta succeeded in 51ms:
nc/cad_read.py:2:# This code modified from iso_read.py and emc2_read.py distriuted with HeeksCAD as of Sep 2010
nc/anilam_crusader_m.py:1:# Preliminary postprocessor support for Anilam Crusader M CNC controller
nc/anilam_crusader_m.py:2:# This code modified from iso.py and emc2.py distriuted with HeeksCAD as of Sep 2010
nc/anilam_crusader_m.py:108:        self.write('(Canned drill cycle ops are not yet supported here on this Anilam Crusader M postprocessor)')
nc/gantry_router.py:2:import emc2
nc/gantry_router.py:4:class Creator(emc2.Creator):
nc/gantry_router.py:6:        emc2.Creator.init(self) 
nc/emc2tap.py:3:import emc2
nc/emc2tap.py:19:class CreatorEMC2tap(emc2.CreatorEMC2):
nc/hm50.py:2:import emc2
nc/hm50.py:4:class Creator(emc2.Creator):
nc/centroid1.py:2:# centroid1.py
nc/centroid1.py:4:# Post Processor for the centroid M40 machine
nc/emc2b.py:32:            self.write( ('(Created with emc2b post processor ' + str(now.strftime("%Y/%m/%d %H:%M")) + ')' + '\n') )
nc/emc2b.py:34:            self.write( ('(Created with emc2b Cutter Radius Compensation post processor ' + str(now.strftime("%Y/%m/%d %H:%M")) + ')' + '\n') )
pycnc/PyProcess.py:56:def HeeksPyPostProcess(include_backplot_processing):
pycnc/PyProcess.py:61:    file_str = (standard_paths.GetTempDir() + "/post.py").replace('\\', '/')
nc/mach3.py:11:		self.write( ('(' + 'GCode created using the HeeksCNC Mach3 post processor' + ')' + '\n') )
pycnc/Program.py:19:        machine_name = config.Read("ProgramMachine", "emc2b")
pycnc/Program.py:200:            file_str = (standard_paths.GetTempDir() + "/post.py").replace('\\', '/')

thinking
**Exploring tag class usage**
exec
/bin/zsh -lc 'cd /tmp/cam_research/heekscnc && rg -n "class CTag|Tag" src/Tag.* src/Profile.cpp src/Program.cpp | head -n 120' in /Users/finn.carlsvi/thetta succeeded in 53ms:
src/Tag.cpp:1:// Tag.cpp
src/Tag.cpp:6:#include "Tag.h"
src/Tag.cpp:13:CTag::CTag(): m_width(10.0),	m_angle(45.0), m_height(4.0){
src/Tag.cpp:18:CTag::CTag( const CTag & rhs ) : HeeksObj(rhs)
src/Tag.cpp:26:CTag& CTag::operator= ( const CTag & rhs )
src/Tag.cpp:41:const wxBitmap &CTag::GetIcon()
src/Tag.cpp:51:void CTag::glCommands(bool select, bool marked, bool no_color)
src/Tag.cpp:65:static void on_set_pos(const double* vt, HeeksObj* object){memcpy(((CTag*)object)->m_pos, vt, 2*sizeof(double));}
src/Tag.cpp:66:static void on_set_width(double value, HeeksObj* object){((CTag*)object)->m_width = value; ((CTag*)object)->WriteDefaultValues();}
src/Tag.cpp:67:static void on_set_angle(double value, HeeksObj* object){((CTag*)object)->m_angle = value; ((CTag*)object)->WriteDefaultValues();}
src/Tag.cpp:68:static void on_set_height(double value, HeeksObj* object){((CTag*)object)->m_height = value; ((CTag*)object)->WriteDefaultValues();}
src/Tag.cpp:70:void CTag::GetProperties(std::list<Property *> *list)
src/Tag.cpp:78:static CTag* object_for_tools = NULL;
src/Tag.cpp:98:void CTag::GetTools(std::list<Tool*>* t_list, const wxPoint* p)
src/Tag.cpp:104:  // This disables the ability to pick a new pos from selected Tag but prevent HeeksCNC from crash.
src/Tag.cpp:109:void CTag::WriteXML(TiXmlNode *root)
src/Tag.cpp:112:	element = heeksCAD->NewXMLElement( "Tag" );
src/Tag.cpp:123:HeeksObj* CTag::ReadFromXMLElement(TiXmlElement* pElem)
src/Tag.cpp:125:	CTag* new_object = new CTag;
src/Tag.cpp:135:void CTag::WriteDefaultValues()

thinking
**Assessing pycam tab requirements**
exec
/bin/zsh -lc "cd /tmp/cam_research/pycam && sed -n '1,220p' pycam/workspace/__init__.py | sed -n '1,220p'" in /Users/finn.carlsvi/thetta succeeded in 52ms:
from enum import Enum


class CollectionName(Enum):
    MODELS = "models"
    TOOLS = "tools"
    PROCESSES = "processes"
    BOUNDS = "bounds"
    TASKS = "tasks"
    TOOLPATHS = "toolpaths"
    EXPORT_SETTINGS = "export_settings"
    EXPORTS = "exports"


class ToolShape(Enum):
    FLAT_BOTTOM = "flat_bottom"
    BALL_NOSE = "ball_nose"
    TORUS = "torus"



thinking
**Listing needed integrations and features**
exec
/bin/zsh -lc "cd /tmp/cam_research/dxf2gcode && rg -n -i \"conventional|climb|inside|outside|pocket|tab|bridge|offset|cc_outside_the_piece|41|42\" source -g '*.py' | head -n 200" in /Users/finn.carlsvi/thetta succeeded in 51ms:
Total output lines: 200

source/epydoc-gen.py:11:# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:24:        self.mytabWidget = QtGui.QTabWidget(self.splitter)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:28:        sizePolicy.setHeightForWidth(self.mytabWidget.sizePolicy().hasHeightForWidth())
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:29:        self.mytabWidget.setSizePolicy(sizePolicy)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:30:        self.mytabWidget.setMinimumSize(QtCore.QSize(200, 0))
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:31:        self.mytabWidget.setObjectName("mytabWidget")
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:32:        self.tab = QtGui.QWidget()
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:33:        self.tab.setObjectName("tab")
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:34:        self.verticalLayout_3 = QtGui.QVBoxLayout(self.tab)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:39:        self.blocksCollapsePushButton = QtGui.QPushButton(self.tab)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:51:        self.blocksExpandPushButton = QtGui.QPushButton(self.tab)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:66:        self.entitiesTreeView = MyTreeView(self.tab)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:74:        self.mytabWidget.addTab(self.tab, "")
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:75:        self.tab_2 = QtGui.QWidget()
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:76:        self.tab_2.setObjectName("tab_2")
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:77:        self.verticalLayout_5 = QtGui.QVBoxLayout(self.tab_2)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:84:        self.layersCollapsePushButton = QtGui.QPushButton(self.tab_2)
source/dxf2gcode_pyQt4_ui/dxf2gcode_pyQt4_ui.py:94:        self.layersExpandPushButton = QtGui.QPushButton(self.tab_2)

thinking
**Planning file count for FreeCAD scripts**
exec
/bin/zsh -lc "cd /tmp/cam_research/FreeCAD && ls src/Mod/CAM/Path/Post/scripts | sed -n '1,200p' | tr '
' ' ' && echo && echo count="'$(ls src/Mod/CAM/Path/Post/scripts | wc -l)' in /Users/finn.carlsvi/thetta succeeded in 53ms:
KineticNCBeamicon2_post.py __init__.py centroid_legacy_post.py centroid_post.py dxf_post.py dynapath_4060_post.py dynapath_post.py estlcam_post.py fablin_post.py fangling_post.py fanuc_post.py gcode_pre.py generic_post.py grbl_legacy_post.py grbl_post.py heidenhain_post.py jtech_post.py linuxcnc_legacy_post.py linuxcnc_post.py mach3_mach4_legacy_post.py mach3_mach4_post.py marlin_post.py masso_g3_post.py nccad_post.py opensbp_post.py opensbp_pre.py philips_post.py rml_post.py rrf_post.py slic3r_pre.py smoothie_post.py snapmaker_post.py svg_post.py test_post.py uccnc_post.py wedm_post.py 
count= 36

thinking
**Finalizing CAM tools rating plan**
codex
Activity snapshot date: **March 9, 2026**.  
Note: your `jscut` and `dxf2gcode` repo names appear outdated; I used active upstreams (`tbfleming/jscut`, `workflo/dxf2gcode`).

## 1) FreeCAD Path/CAM Workbench
1. **Overview**: Full CAD/CAM platform; CAM workbench in Python/C++; **LGPL-2.1**; https://github.com/FreeCAD/FreeCAD  
2. **Activity**: Very active (last push **2026-03-09**), ~960 contributors, many releases (84). Large community.  
3. **Profile cuts**: Strong 2D profile support (`Inside/Outside`, direction `CW/CCW`, compensation options).  
4. **Tabs**: Strong via Holding Tags dressup; manual and generated tags; width/height/angle/spacing/count controls.  
5. **Pockets**: Strong pocketing (Pocket op + Adaptive op; zig-zag/adaptive behaviors).  
6. **G-code**: Very broad post support (GRBL, LinuxCNC, Fanuc, Mach3/4, Masso, Marlin, Smoothie, etc.).  
7. **Integration**: Weak as embeddable library (monolith), but strong scripting/Python automation; Rust integration likely via process/Python bridge, not direct linking.  
8. **Strengths**: Feature complete, mature, robust post ecosystem.  
9. **Weaknesses**: Heavy dependency surface, complex extraction.  
10. **Rating**: **4/5**.

## 2) BlenderCAM (Fabex)
1. Blender add-on CAM in Python; **GPL-3.0**; https://github.com/vilemduha/blendercam  
2. Active (last push **2026-01-01**), ~51 contributors, releases exist.  
3. Profile: Yes (inside/outside/on-line profile strategies; climb/conventional options).  
4. Tabs: Yes (“Bridges/Tabs”), automated with configurable width/height/min count/distance.  
5. Pocket: Yes (pocket ops; multiple 2D/3D strategies).  
6. G-code: Multiple post processors (GRBL, Mach3, EMC/LinuxCNC variants, etc.).  
7. Integration: Hard as library (tied to Blender runtime/data model).  
8. Strengths: Rich CAM for routing workflows.  
9. Weaknesses: Blender dependency is heavy for Rust service integration.  
10. Rating: **3/5**.

## 3) PyCAM
1. Python CAM for 3-axis; **GPL-3.0**; https://github.com/SebKuzminsky/pycam  
2. Moderate maturity, slower recent development; ~23 contributors; multiple historical releases.  
3. Profile: Yes (contour strategies, inside/along/around boundary modes).  
4. Tabs: Yes (“support_bridges” with grid/distributed layouts).  
5. Pocket: Yes (grid/spiral style patterns; contour/slice/surface strategies).  
6. G-code: Limited dialect support in-tree (notably LinuxCNC exporter).  
7. Integration: Better than GUI-heavy tools (Python package/code), but still Python-first.  
8. Strengths: 3-axis focus, scriptable flow model.  
9. Weaknesses: Narrow post ecosystem; older architecture.  
10. Rating: **3/5**.

## 4) OpenCAMLib
1. CAM algorithm library (C++ + Python + Node/WASM bindings); **LGPL-2.1**; https://github.com/aewallin/opencamlib  
2. Mature algorithms, moderate activity, ~31 contributors.  
3. Profile: Not a full 2D profile CAM app; provides geometric toolpath primitives (drop-cutter/waterline), not turnkey profiling workflow.  
4. Tabs: No native holding-tab operation.  
5. Pocket: No full 2D pocket workflow out of box (building block level).  
6. G-code: No full post-processing stack by default.  
7. Integration: Excellent for Rust FFI to C++ core if you build missing orchestration yourself.  
8. Strengths: Library-first design, good for algorithm reuse.  
9. Weaknesses: You must implement profile/tab/pocket orchestration and post layer yourself.  
10. Rating: **2/5** for your immediate needs (higher if you want to build core CAM in-house).

## 5) DerpCAM
1. 2.5D CAM for 3-axis routers/mills; Python; **GPL-3.0**; https://github.com/kfoltman/DerpCAM  
2. Smaller project, low contributor count (primarily single maintainer).  
3. Profile: Strong (inside/outside slots, side milling, climb/conventional).  
4. Tabs: Strong (automatic + manual placement).  
5. Pocket: Strong for 2D/2.5D (islands, multiple strategies incl. HSM/trochoidal notes).  
6. G-code: GRBL and LinuxCNC dialects.  
7. Integration: Good practical fit via Python API; Rust integration via embedded Python/subprocess likely straightforward.  
8. Strengths: Directly aligned with your 2D nesting post-process needs.  
9. Weaknesses: Small bus factor; limited post dialect breadth.  
10. Rating: **4/5**.

## 6) Kiri:Moto (GridSpace/grid-apps)
1. Browser CAM/slicer platform in JS; **MIT**; https://github.com/GridSpace/grid-apps  
2. Active and mature (last push near **2026-03**), ~38 contributors, frequent releases.  
3. Profile: Yes (contour/path-inside/path-outside operations).  
4. Tabs: Yes (tab generation and tab-hopping behavior present).  
5. Pocket: Yes (pocket operations and ongoing improvements).  
6. G-code: Device/profile-driven generation; broad practical machine coverage via profiles.  
7. Integration: Good if you accept JS runtime; exposes slicing/G-code APIs (`engine.html`, frame API).  
8. Strengths: Modern architecture, API-oriented, active.  
9. Weaknesses: JS/web stack integration overhead in Rust backend.  
10. Rating: **4/5**.

## 7) jscut
1. Browser-based 2D CAM in JS; **GPL-3.0**; https://github.com/tbfleming/jscut  
2. Older/stable but low recent velocity; small contributor base.  
3. Profile: Yes (Inside/Outside ops; climb/conventional control).  
4. Tabs: Yes (tab UI + tab geometry processing; mostly user/manual driven).  
5. Pocket: Yes (Pocket op).  
6. G-code: Simple generic G-code generator, limited post sophistication.  
7. Integration: Good as lightweight JS CAM engine; easier extraction than large GUI systems.  
8. Strengths: Simple 2D CAM core matching nesting output geometry.  
9. Weaknesses: Aging project; limited advanced strategies and ecosystem.  
10. Rating: **3/5**.

## 8) OpenBuilds CAM
1. Web CAM app (JS); **AGPL-3.0**; https://github.com/OpenBuilds/OpenBuilds-CAM  
2. Moderate activity; small maintainer set.  
3. Profile: Yes (`CNC: Vector` no offset / inside / outside).  
4. Tabs: Yes (tab depth/width/spacing and automatic placement logic in worker).  
5. Pocket: Yes (`CNC: Pocket` op).  
6. G-code: Primarily GRBL/OpenBuilds ecosystem oriented; practical CNC router output.  
7. Integration: Possible by lifting worker logic; but app-centric codebase and AGPL constraints matter.  
8. Strengths: Directly supports your 3 needed operations.  
9. Weaknesses: AGPL obligations; not designed as clean embeddable library.  
10. Rating: **4/5** technically, **3/5** if AGPL is a blocker.

## 9) CAMotics
1. Primarily CNC simulation/verification; C++; GPL family; https://github.com/CauldronDevelopmentLLC/CAMotics  
2. Active simulator project, moderate community.  
3. Profile: Not intended as primary profile toolpath generator from 2D nesting geometry.  
4. Tabs: No practical holding-tab CAM workflow.  
5. Pocket: Not a production pocket CAM planner for your use case.  
6. G-code: Excellent parsing/simulation of G-code, not post-focused CAM authoring.  
7. Integration: Good as verifier/simulator stage, poor as CAM generation stage.  
8. Strengths: Validation and collision/risk reduction.  
9. Weaknesses: Doesn’t solve your core post-processing generation requirements.  
10. Rating: **1/5** for CAM generation (but useful adjunct).

## 10) HeeksCNC
1. Older CAM add-on for HeeksCAD; C++/Python; BSD-style license text in repo; https://github.com/Heeks/heekscnc  
2. Mature but low recent activity (last push around **2021**).  
3. Profile: Yes (left/outside, right/inside, on; climb/conventional).  
4. Tabs: Yes (Tag objects and profile tag-adding mode, largely manual).  
5. Pocket: Yes (zig-zag options, entry styles including helical/ramp/plunge).  
6. G-code: Post-processor scripts exist (emc2/mach3/centroid/etc.).  
7. Integration: Possible but legacy architecture and aging dependencies are substantial.  
8. Strengths: Has your three operation classes.  
9. Weaknesses: Legacy stack and maintenance risk.  
10. Rating: **2/5**.

## 11) DXF2GCODE
1. DXF-to-G-code converter; Python/Qt; **GPL-3.0**; https://github.com/workflo/dxf2gcode  
2. Very low activity (last commit around **2015** in this fork).  
3. Profile: Yes for contour/offset-style cutting.  
4. Tabs: No meaningful holding-tab workflow found.  
5. Pocket: Not a modern pocket-clearing CAM system.  
6. G-code: Postprocessor config system exists; focused DXF conversion pipeline.  
7. Integration: Legacy GUI-oriented code; extraction effort high vs payoff.  
8. Strengths: Basic contour conversion.  
9. Weaknesses: Stale, limited for modern nesting CAM needs.  
10. Rating: **2/5**.

## 12) PathCAM
1. C# 2.5D router toolpath app; **GPL-3.0**; https://github.com/xenovacivus/PathCAM  
2. Low-maintainer project; sporadic updates.  
3. Profile: Yes (perimeter toolpaths with inside/outside handling in planner).  
4. Tabs: Yes (Tab class with auto spacing/min tabs + avoidance in path generation).  
5. Pocket: Limited; more perimeter/rout-area behavior than rich pocket strategy set.  
6. G-code: Simple export and some direct machine connectivity (incl. GRBL mention).  
7. Integration: Potentially extractable C# planning code, but not Rust-native and limited ecosystem.  
8. Strengths: Built-in tab-aware path planning.  
9. Weaknesses: Narrow scope, GPL, smaller community.  
10. Rating: **2/5**.

---

## Practical shortlist for your Rust nesting pipeline
1. **DerpCAM** (best direct feature fit + Python API).  
2. **Kiri:Moto** (active, capable, API-friendly if JS runtime is acceptable).  
3. **OpenBuilds CAM** (feature fit strong; check AGPL implications).  
4. **FreeCAD CAM** (most capable, but heavy for embedding).

## Sources
- FreeCAD CAM modules and posts: https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/CAM  
- BlenderCAM docs/features/posts: https://github.com/vilemduha/blendercam  
- PyCAM workspace/enums/exporters: https://github.com/SebKuzminsky/pycam  
- OpenCAMLib docs/manual/bindings: https://github.com/aewallin/opencamlib  
- DerpCAM README (features/API): https://github.com/kfoltman/DerpCAM  
- Kiri:Moto/grid-apps README + release notes: https://github.com/GridSpace/grid-apps  
- jscut CAM API and UI ops: https://github.com/tbfleming/jscut  
- OpenBuilds CAM worker/toolpath ops: https://github.com/OpenBuilds/OpenBuilds-CAM  
- CAMotics README: https://github.com/CauldronDevelopmentLLC/CAMotics  
- HeeksCNC profile/pocket/tag/post files: https://github.com/Heeks/heekscnc  
- DXF2GCODE source/post config: https://github.com/workflo/dxf2gcode  
- PathCAM README + Router/Paths code: https://github.com/xenovacivus/PathCAM
tokens used
147,017
Activity snapshot date: **March 9, 2026**.  
Note: your `jscut` and `dxf2gcode` repo names appear outdated; I used active upstreams (`tbfleming/jscut`, `workflo/dxf2gcode`).

## 1) FreeCAD Path/CAM Workbench
1. **Overview**: Full CAD/CAM platform; CAM workbench in Python/C++; **LGPL-2.1**; https://github.com/FreeCAD/FreeCAD  
2. **Activity**: Very active (last push **2026-03-09**), ~960 contributors, many releases (84). Large community.  
3. **Profile cuts**: Strong 2D profile support (`Inside/Outside`, direction `CW/CCW`, compensation options).  
4. **Tabs**: Strong via Holding Tags dressup; manual and generated tags; width/height/angle/spacing/count controls.  
5. **Pockets**: Strong pocketing (Pocket op + Adaptive op; zig-zag/adaptive behaviors).  
6. **G-code**: Very broad post support (GRBL, LinuxCNC, Fanuc, Mach3/4, Masso, Marlin, Smoothie, etc.).  
7. **Integration**: Weak as embeddable library (monolith), but strong scripting/Python automation; Rust integration likely via process/Python bridge, not direct linking.  
8. **Strengths**: Feature complete, mature, robust post ecosystem.  
9. **Weaknesses**: Heavy dependency surface, complex extraction.  
10. **Rating**: **4/5**.

## 2) BlenderCAM (Fabex)
1. Blender add-on CAM in Python; **GPL-3.0**; https://github.com/vilemduha/blendercam  
2. Active (last push **2026-01-01**), ~51 contributors, releases exist.  
3. Profile: Yes (inside/outside/on-line profile strategies; climb/conventional options).  
4. Tabs: Yes (“Bridges/Tabs”), automated with configurable width/height/min count/distance.  
5. Pocket: Yes (pocket ops; multiple 2D/3D strategies).  
6. G-code: Multiple post processors (GRBL, Mach3, EMC/LinuxCNC variants, etc.).  
7. Integration: Hard as library (tied to Blender runtime/data model).  
8. Strengths: Rich CAM for routing workflows.  
9. Weaknesses: Blender dependency is heavy for Rust service integration.  
10. Rating: **3/5**.

## 3) PyCAM
1. Python CAM for 3-axis; **GPL-3.0**; https://github.com/SebKuzminsky/pycam  
2. Moderate maturity, slower recent development; ~23 contributors; multiple historical releases.  
3. Profile: Yes (contour strategies, inside/along/around boundary modes).  
4. Tabs: Yes (“support_bridges” with grid/distributed layouts).  
5. Pocket: Yes (grid/spiral style patterns; contour/slice/surface strategies).  
6. G-code: Limited dialect support in-tree (notably LinuxCNC exporter).  
7. Integration: Better than GUI-heavy tools (Python package/code), but still Python-first.  
8. Strengths: 3-axis focus, scriptable flow model.  
9. Weaknesses: Narrow post ecosystem; older architecture.  
10. Rating: **3/5**.

## 4) OpenCAMLib
1. CAM algorithm library (C++ + Python + Node/WASM bindings); **LGPL-2.1**; https://github.com/aewallin/opencamlib  
2. Mature algorithms, moderate activity, ~31 contributors.  
3. Profile: Not a full 2D profile CAM app; provides geometric toolpath primitives (drop-cutter/waterline), not turnkey profiling workflow.  
4. Tabs: No native holding-tab operation.  
5. Pocket: No full 2D pocket workflow out of box (building block level).  
6. G-code: No full post-processing stack by default.  
7. Integration: Excellent for Rust FFI to C++ core if you build missing orchestration yourself.  
8. Strengths: Library-first design, good for algorithm reuse.  
9. Weaknesses: You must implement profile/tab/pocket orchestration and post layer yourself.  
10. Rating: **2/5** for your immediate needs (higher if you want to build core CAM in-house).

## 5) DerpCAM
1. 2.5D CAM for 3-axis routers/mills; Python; **GPL-3.0**; https://github.com/kfoltman/DerpCAM  
2. Smaller project, low contributor count (primarily single maintainer).  
3. Profile: Strong (inside/outside slots, side milling, climb/conventional).  
4. Tabs: Strong (automatic + manual placement).  
5. Pocket: Strong for 2D/2.5D (islands, multiple strategies incl. HSM/trochoidal notes).  
6. G-code: GRBL and LinuxCNC dialects.  
7. Integration: Good practical fit via Python API; Rust integration via embedded Python/subprocess likely straightforward.  
8. Strengths: Directly aligned with your 2D nesting post-process needs.  
9. Weaknesses: Small bus factor; limited post dialect breadth.  
10. Rating: **4/5**.

## 6) Kiri:Moto (GridSpace/grid-apps)
1. Browser CAM/slicer platform in JS; **MIT**; https://github.com/GridSpace/grid-apps  
2. Active and mature (last push near **2026-03**), ~38 contributors, frequent releases.  
3. Profile: Yes (contour/path-inside/path-outside operations).  
4. Tabs: Yes (tab generation and tab-hopping behavior present).  
5. Pocket: Yes (pocket operations and ongoing improvements).  
6. G-code: Device/profile-driven generation; broad practical machine coverage via profiles.  
7. Integration: Good if you accept JS runtime; exposes slicing/G-code APIs (`engine.html`, frame API).  
8. Strengths: Modern architecture, API-oriented, active.  
9. Weaknesses: JS/web stack integration overhead in Rust backend.  
10. Rating: **4/5**.

## 7) jscut
1. Browser-based 2D CAM in JS; **GPL-3.0**; https://github.com/tbfleming/jscut  
2. Older/stable but low recent velocity; small contributor base.  
3. Profile: Yes (Inside/Outside ops; climb/conventional control).  
4. Tabs: Yes (tab UI + tab geometry processing; mostly user/manual driven).  
5. Pocket: Yes (Pocket op).  
6. G-code: Simple generic G-code generator, limited post sophistication.  
7. Integration: Good as lightweight JS CAM engine; easier extraction than large GUI systems.  
8. Strengths: Simple 2D CAM core matching nesting output geometry.  
9. Weaknesses: Aging project; limited advanced strategies and ecosystem.  
10. Rating: **3/5**.

## 8) OpenBuilds CAM
1. Web CAM app (JS); **AGPL-3.0**; https://github.com/OpenBuilds/OpenBuilds-CAM  
2. Moderate activity; small maintainer set.  
3. Profile: Yes (`CNC: Vector` no offset / inside / outside).  
4. Tabs: Yes (tab depth/width/spacing and automatic placement logic in worker).  
5. Pocket: Yes (`CNC: Pocket` op).  
6. G-code: Primarily GRBL/OpenBuilds ecosystem oriented; practical CNC router output.  
7. Integration: Possible by lifting worker logic; but app-centric codebase and AGPL constraints matter.  
8. Strengths: Directly supports your 3 needed operations.  
9. Weaknesses: AGPL obligations; not designed as clean embeddable library.  
10. Rating: **4/5** technically, **3/5** if AGPL is a blocker.

## 9) CAMotics
1. Primarily CNC simulation/verification; C++; GPL family; https://github.com/CauldronDevelopmentLLC/CAMotics  
2. Active simulator project, moderate community.  
3. Profile: Not intended as primary profile toolpath generator from 2D nesting geometry.  
4. Tabs: No practical holding-tab CAM workflow.  
5. Pocket: Not a production pocket CAM planner for your use case.  
6. G-code: Excellent parsing/simulation of G-code, not post-focused CAM authoring.  
7. Integration: Good as verifier/simulator stage, poor as CAM generation stage.  
8. Strengths: Validation and collision/risk reduction.  
9. Weaknesses: Doesn’t solve your core post-processing generation requirements.  
10. Rating: **1/5** for CAM generation (but useful adjunct).

## 10) HeeksCNC
1. Older CAM add-on for HeeksCAD; C++/Python; BSD-style license text in repo; https://github.com/Heeks/heekscnc  
2. Mature but low recent activity (last push around **2021**).  
3. Profile: Yes (left/outside, right/inside, on; climb/conventional).  
4. Tabs: Yes (Tag objects and profile tag-adding mode, largely manual).  
5. Pocket: Yes (zig-zag options, entry styles including helical/ramp/plunge).  
6. G-code: Post-processor scripts exist (emc2/mach3/centroid/etc.).  
7. Integration: Possible but legacy architecture and aging dependencies are substantial.  
8. Strengths: Has your three operation classes.  
9. Weaknesses: Legacy stack and maintenance risk.  
10. Rating: **2/5**.

## 11) DXF2GCODE
1. DXF-to-G-code converter; Python/Qt; **GPL-3.0**; https://github.com/workflo/dxf2gcode  
2. Very low activity (last commit around **2015** in this fork).  
3. Profile: Yes for contour/offset-style cutting.  
4. Tabs: No meaningful holding-tab workflow found.  
5. Pocket: Not a modern pocket-clearing CAM system.  
6. G-code: Postprocessor config system exists; focused DXF conversion pipeline.  
7. Integration: Legacy GUI-oriented code; extraction effort high vs payoff.  
8. Strengths: Basic contour conversion.  
9. Weaknesses: Stale, limited for modern nesting CAM needs.  
10. Rating: **2/5**.

## 12) PathCAM
1. C# 2.5D router toolpath app; **GPL-3.0**; https://github.com/xenovacivus/PathCAM  
2. Low-maintainer project; sporadic updates.  
3. Profile: Yes (perimeter toolpaths with inside/outside handling in planner).  
4. Tabs: Yes (Tab class with auto spacing/min tabs + avoidance in path generation).  
5. Pocket: Limited; more perimeter/rout-area behavior than rich pocket strategy set.  
6. G-code: Simple export and some direct machine connectivity (incl. GRBL mention).  
7. Integration: Potentially extractable C# planning code, but not Rust-native and limited ecosystem.  
8. Strengths: Built-in tab-aware path planning.  
9. Weaknesses: Narrow scope, GPL, smaller community.  
10. Rating: **2/5**.

---

## Practical shortlist for your Rust nesting pipeline
1. **DerpCAM** (best direct feature fit + Python API).  
2. **Kiri:Moto** (active, capable, API-friendly if JS runtime is acceptable).  
3. **OpenBuilds CAM** (feature fit strong; check AGPL implications).  
4. **FreeCAD CAM** (most capable, but heavy for embedding).

## Sources
- FreeCAD CAM modules and posts: https://github.com/FreeCAD/FreeCAD/tree/main/src/Mod/CAM  
- BlenderCAM docs/features/posts: https://github.com/vilemduha/blendercam  
- PyCAM workspace/enums/exporters: https://github.com/SebKuzminsky/pycam  
- OpenCAMLib docs/manual/bindings: https://github.com/aewallin/opencamlib  
- DerpCAM README (features/API): https://github.com/kfoltman/DerpCAM  
- Kiri:Moto/grid-apps README + release notes: https://github.com/GridSpace/grid-apps  
- jscut CAM API and UI ops: https://github.com/tbfleming/jscut  
- OpenBuilds CAM worker/toolpath ops: https://github.com/OpenBuilds/OpenBuilds-CAM  
- CAMotics README: https://github.com/CauldronDevelopmentLLC/CAMotics  
- HeeksCNC profile/pocket/tag/post files: https://github.com/Heeks/heekscnc  
- DXF2GCODE source/post config: https://github.com/workflo/dxf2gcode  
- PathCAM README + Router/Paths code: https://github.com/xenovacivus/PathCAM
