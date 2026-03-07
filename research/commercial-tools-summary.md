# Commercial Nesting Software Tools: Comprehensive Analysis

This document provides an in-depth analysis of the major commercial nesting software tools,
their algorithms, features, strengths, weaknesses, and relevance to the Thetta project.

---

## Table of Contents

1. [SigmaNEST](#1-sigmanest)
2. [ProNest (Hypertherm)](#2-pronest-hypertherm)
3. [TruTops Boost / TruNest (TRUMPF)](#3-trutops-boost--trunest-trumpf)
4. [NestFab](#4-nestfab)
5. [OptiNest (Boole & Partners)](#5-optinest-boole--partners)
6. [Alma Act/Nesting (AlmaCAM / PowerNest)](#6-alma-actnesting-almacam--powernest)
7. [Lantek Expert](#7-lantek-expert)
8. [NestLib (HCL Technologies / Geometric)](#8-nestlib-hcl-technologies--geometric)
9. [Astra R-Nesting / Astra S-Nesting (Technos)](#9-astra-r-nesting--astra-s-nesting-technos)
10. [JetCAM](#10-jetcam)
11. [Commercial vs. Open-Source Nesting](#11-commercial-vs-open-source-nesting)
12. [Key Takeaways for Thetta](#12-key-takeaways-for-thetta)

---

## 1. SigmaNEST

### Overview

SigmaNEST is the world's leading commercial nesting software, now part of Sandvik Manufacturing
Solutions. With over 21,000 installations globally, it supports all major brands of laser, plasma,
punch, router, waterjet, tube, and pressbrake machines. It is consistently ranked as the top
CAD/CAM nesting solution in the industry.

### Nesting Algorithm

- **True-shape nesting** using proprietary algorithms (almost certainly NFP-based internally)
- Advanced algorithms optimize placement using **part rotation, mirroring, and adaptive strategies**
- **Dynamic evaluation**: continuously assesses part geometry and material properties to generate
  the most efficient layouts
- **Learning capability**: the algorithm refines its approach with each nesting scenario, adapting
  to variations in part sizes and shapes
- Adjusts part spacing based on **kerf width** of the cutting tool
- Accounts for **material thickness and grain direction**
- Customers typically save **>4% on material** when switching to SigmaNEST

### Features

- **Automatic and manual true-shape nesting**
- **Part-in-part nesting** (smaller parts nested inside cutouts of larger parts)
- **Common-line cutting** support
- **Remnant nesting** with First-In First-Out approach for lean inventory
- **Multi-head machine** support with adaptive nesting
- **Complex machine support**: hold-downs, clamps, repositioning, bevel cutting
- **Right-angle shear** and drop-door part ejection
- **Real-time layout updates** as production parameters change
- **Estimating and quoting** module for part pricing
- **NC toolpath generation** and optimization
- **Shop floor integration** (SigmaSCHEDULE, SigmaMRP)

### Input/Output Formats

- **Native import**: DXF, DWG, DSTV, STEP
- **With CAD Import module**: IGES, SolidWorks, Inventor, Solid Edge/NX, PTC Creo, PDF/images
- **Automatic error correction** for DXF, DWG, CDL, IGES, DSTV, STEP, G-code, NC ESSI
- Automatic calculation of part area, weight, cutting time, cost on import

### Strengths

- Industry-leading material utilization (>4% improvement typical)
- Extremely broad machine compatibility (all major cutting technologies)
- Comprehensive production workflow (CAD to NC to shop floor)
- Intuitive interface with relatively fast learning curve for basic operations
- Strong technical support and customer service
- Part of Sandvik -- long-term stability and investment

### Weaknesses

- **Steep learning curve** for advanced features -- substantial training investment
- **High cost** -- potential barrier for smaller businesses
- Cannot nest multiple sheets of different thicknesses simultaneously
- Pricing not transparent (quote-based)

### Pricing Model

- Starts at approximately **$199/month**
- Available as **perpetual license** or **annual subscription**
- Quote-based pricing depending on modules, seats, machine types, support level
- Users note cost is "slightly higher but well worth it"

### Relevance to Thetta

- SigmaNEST's adaptive/learning nesting approach suggests value in iterative refinement strategies
- Their kerf-width-aware spacing is a feature Thetta should implement
- Part-in-part nesting is a high-value feature for material optimization
- The fact that >4% material savings is considered impressive tells us about realistic benchmarks

---

## 2. ProNest (Hypertherm)

### Overview

ProNest is one of the world's leading CAD/CAM nesting solutions, developed by Hypertherm (the
cutting experts). Serving tens of thousands of customers since 1984, it optimizes performance
for plasma, laser, waterjet, and oxyfuel cutting machines. ProNest is available in a full
advanced version and a lighter ProNest LT version.

### Nesting Algorithm

- Uses **one or more mathematical nesting algorithms** that automatically add shapes to nests
- Parts are **rotated and moved** for best overall fit and tightest material usage
- **IntelliChoice** mode: ProNest automatically determines the best nesting strategy
- **Nesting System Optimization** module: calculates every possible combination of parts, plates,
  remnants, and strategies to find the best outcome
- Accounts for different jobs with **multiple combinations** of parts, plates, and strategies
- Users report **~5% material utilization savings**, translating to **~$50,000/year** in material

### Features

- **Automatic nesting** with selectable strategies (speed vs. utilization tradeoff)
- **Common-line cutting** (quad CLC, CLC Array, Advanced CLC for dissimilar parts)
- **Chain cutting** (linking multiple profiles into one continuous cut)
- **Bridge cutting** (thin webs between parts to prevent tip-ups)
- **Collision avoidance** (automatic adjustment of leads, traversals, cut sequence)
- **Skeleton cut-up** for easy removal of waste material
- **SureCut technologies**: True Hole, Rapid Part, PlateSaver, True Bevel
- **Variable shape profiles** (geometric shapes without CAD file)
- **Cut sequence** control (automatic, semi-automatic, manual)
- **Production Manager** with real-time machine status dashboard
- **ERP/MRP integration** with any third-party system
- **Tekla PowerFab and STRUMIS connectors** for structural steel

### Input/Output Formats

- Imports from **all major CAD systems** (DXF, DWG, and others)
- Built-in **2D CAD editor**
- NC code generation for plasma, laser, waterjet, oxyfuel

### Strengths

- Tight integration with Hypertherm cutting systems (SureCut technologies)
- Very strong **common-line cutting** capabilities (quad, array, dissimilar)
- Comprehensive production management beyond just nesting
- **Collision avoidance** included by default
- Can reduce programming time from **hours to minutes**
- Cut time reduction of **up to 50%** with specialized techniques
- Affordable entry point with ProNest LT subscription

### Weaknesses

- Best features require Hypertherm cutting hardware for full benefit
- Full version pricing not transparent (quote-based)
- ProNest LT limited to conventional plasma and oxyfuel (no laser/HD plasma)
- Less independent than SigmaNEST (Hypertherm ecosystem bias)

### Pricing Model

- **ProNest (full)**: One-time perpetual license, quote-based pricing
- **ProNest LT**: Monthly subscription starting at **$29.99/month**
- ProNest LT Unlimited: **$485.89** for 1-3 year subscription
- 7-day free trial available for ProNest LT
- Subscriptions include support, training, and updates

### Relevance to Thetta

- The **IntelliChoice** concept (auto-selecting the best strategy) is worth emulating
- **Nesting System Optimization** (trying all combinations) is brute-force but effective
- Common-line cutting for dissimilar parts is an advanced feature Thetta could target
- Chain/bridge cutting concepts show the importance of cut-path-aware nesting

---

## 3. TruTops Boost / TruNest (TRUMPF)

### Overview

TruTops Boost is TRUMPF's proprietary software for 2D/3D design and programming of laser,
punching, punch-laser, and bending machines. It is an OEM solution -- the same company builds
the machines and develops the software. Note: **TruNest** is actually a separate third-party
nesting product, not developed by TRUMPF. TRUMPF's own nesting is integrated within TruTops
Boost via the **Lean Nest** processor.

### Nesting Algorithm

- **Lean Nest** nesting processor with optimized nesting algorithms
- Takes nesting parameters into account: start corner, part distances, sheet margins
- **Scrap-optimized nesting** algorithm with no time limit for maximum sheet utilization
- **TwinLine blocks** for paired/mirrored part nesting in laser cutting
- All processing on parts and sheets is taken into account when nesting
- Automatic **scrap skeleton separating cuts**

### Features

- **One-click "Boost" button**: automates design-to-NC-program creation
- **TecZone Cut**: nesting job optimization with a few clicks
- **HomeZone**: overview of all work steps from design to cutting/punching
- **Nanojoint function**: minimizes marking on tacked parts
- **Smart Collision Prevention (SCP 4.0)**: avoids collisions during part removal
- **3D design and unfolding** integrated
- **Bending program** integration
- Supports laser, punching, punch-laser, and bending machines

### Input/Output Formats

- Integrated 2D/3D design environment
- Imports standard CAD formats
- NC code generation for TRUMPF machines
- Primarily designed for TRUMPF machine ecosystem

### Strengths

- **Seamless integration** with TRUMPF machines (best-in-class for TRUMPF users)
- **One-click automation** dramatically reduces programming time
- Scrap-optimized nesting with unlimited computation time option
- Smart collision prevention for automated part removal
- Combines design, nesting, and NC programming in one tool

### Weaknesses

- **TRUMPF machines only** -- not usable with other manufacturers
- **Vendor lock-in** to TRUMPF ecosystem
- Less flexibility than independent nesting software
- Pricing not transparent; bundled with machine purchase
- Limited third-party integration

### Pricing Model

- **Not publicly listed** -- typically bundled with TRUMPF machine purchases
- License management through TRUMPF's proprietary system
- Contact TRUMPF directly for quotes

### Relevance to Thetta

- The **unlimited computation time** option for better results is a good UX pattern
- Scrap skeleton separating cuts are a valuable post-nesting feature
- The one-click automation philosophy (from geometry to result) is a good design target
- TwinLine block concept (paired/mirrored parts) is worth implementing

---

## 4. NestFab

### Overview

NestFab is described as the "world's most effective automatic nesting software," designed for
sheet metal fabrication, woodworking, CNC machining, and more. It emphasizes ease of use and
has been developed over 25+ years of industrial experience. Available in Standard and Ultra
Performance editions.

### Nesting Algorithm

- **Intelligent optimization** with advanced algorithms
- **Ultra Performance (UP) nesting engine**: claims 5-20% material savings over other automatic
  nesting solutions
- Supports **exact part and sheet spacings** including 0 gap
- **Free rotation** with configurable tilt tolerance for grained materials (e.g., 180 deg with
  +/- 3 degrees tolerance)
- **Best sheet size selection** from available stock
- Three nesting modes: Maximum efficiency / Balanced / Repeat
- Favors **repeated nests** where possible to reduce programming time
- Supports **common cuts** to reduce cutting time

### Features

- **Automatic nesting** with full control over quantities, sheet sizes, part rotation
- **Interactive CAD importer** (smart enough to ignore borders/title blocks)
- Multiple parts from single CAD file (no need to split)
- **Full automation processes** for around-the-clock production
- **Nesting reports** for production team (up to 40% faster part identification)
- **CSV/XML integration** for production planning system imports
- Layer and color filtering for selective imports
- Linked previews for verification against original drawings

### Input/Output Formats

- **Industry-standard CAD formats** (DXF and others, up to latest versions)
- Export to industry-standard CAD formats with extensive export options
- CSV and XML import for production planning data
- Command-line integration for automation

### Strengths

- **Clean, modern, easy-to-use** interface -- no advanced expertise required
- Ultra Performance engine with claimed best-in-class material efficiency
- **Tilt tolerance** for grain direction (e.g., 180 deg +/- 3 deg) is a unique feature
- Transparent pricing (rare in this market)
- Good automation capabilities for batch processing
- 7-day trial available

### Weaknesses

- Less well-known than SigmaNEST or ProNest
- Fewer integrations with ERP/MRP systems
- No built-in CAD editor
- Limited machine-specific postprocessor support
- Smaller user community

### Pricing Model

- **Annual subscription**: $2,500/year (extra seats $1,250/year)
- **One-time purchase**: $8,250 (extra seats $3,495)
- Ultra Performance edition available as add-on
- 7-day free trial

### Relevance to Thetta

- The **tilt tolerance** concept (allowing small angular deviations for grain) is clever and
  worth implementing in Thetta
- Three nesting modes (max efficiency / balanced / repeat) is a good UX pattern
- The clean, modern interface philosophy aligns with Thetta's goals
- Transparent pricing model is a good business reference

---

## 5. OptiNest (Boole & Partners)

### Overview

OptiNest is a general-shapes nesting software developed by BOOLE & PARTNERS, a French
computer-assisted optimization specialist since 1988. Built on the CalepiLib functions library,
it has several thousand licenses in use in over 30 countries. Primarily used in woodworking but
applicable to any sheet material cutting.

### Nesting Algorithm

- Based on BOOLE & PARTNERS' **CalepiLib functions library**
- Default mode: smallest **bounding rectangle** approach for irregular shapes
- Optional **true-shape nesting** that fits shapes within shapes with minimal spacing
- **Adaptable algorithms** with configurable parameters:
  - **Calculation depth**: controls optimization thoroughness vs. speed
  - **Smoothing factor**: controls how shape complexity is handled
  - **Pieces grouping**: how identical parts are clustered
  - **Propagation mode**: how single parts produced in large quantities are placed
- Propagation places a part grouped with copies of itself, then propagated along 2 axes
- Supports 4 rotation levels and flipping

### Features

- **DXF import/export** with built-in graphic editor for drawing parts
- **Cut-out recovery**: nests parts inside cut-outs of other parts
- **Multi-stock and multi-format**: handles multiple panel stocks simultaneously
- **Drilling/boring management** (holes as specific tooling type)
- **Continuous grain matching** (grouped parts placed in sequence for grain continuity)
- **Customizable labels** with QR code, barcode, touch screen support
- **Stock management** with existing stock levels and usable off-cut settings
- **Integration with PolyBoard** cabinet design software
- Reports with material cost, off-cut reuse statistics, machining load

### Input/Output Formats

- **DXF import/export** (polylines, arcs, lines)
- Parts can be drawn directly in OptiNest's graphic editor
- Export to CAD/CAM software for CNC transfer

### Strengths

- **Highly configurable** algorithm parameters for different optimization priorities
- **Cut-out recovery** (part-in-part) included
- **Continuous grain matching** is unique and valuable for wood/aesthetic materials
- Propagation mode is smart for large-quantity single-part production
- Good for woodworking with PolyBoard integration
- Mature product (35+ years of development)

### Weaknesses

- DXF-only for import (limited format support)
- True-shape nesting only available as optional add-on
- Primarily woodworking-focused (less suited for sheet metal)
- UI appears dated compared to modern tools
- Limited CNC postprocessor support
- Light version limited to 10-vertex polygons

### Pricing Model

- Not publicly listed
- Free trial/demo available
- Contact Boole & Partners or authorized resellers for quotes

### Relevance to Thetta

- The **configurable algorithm parameters** (depth, smoothing, grouping, propagation) provide a
  good model for exposable tuning knobs
- **Propagation mode** (self-grouping then 2-axis propagation) is an efficient heuristic for
  high-quantity single-part nesting
- **Continuous grain matching** is a niche but valuable feature
- The CalepiLib library approach (reusable algorithm library) mirrors Thetta's architecture goals

---

## 6. Alma Act/Nesting (AlmaCAM / PowerNest)

### Overview

Alma is a French employee-owned cooperative founded in 1979 with 45+ years in manufacturing
software. They are unique in the industry as the **only player to develop in-house nesting
algorithms AND deliver them both as software components (PowerNest library) and as ready-to-use
CAM software (AlmaCAM/act/cut)**. R&D intensity is ~25% of revenue. Their nesting technology
is used across sheet metal, packaging, plastics, shipbuilding, automotive, and aerospace.

### Nesting Algorithm

- **True-shape nesting** using proprietary algorithms (details not publicly disclosed)
- Algorithms use **real/true shapes** rather than bounding boxes
- **Automatically selects** among several domain-specific algorithms based on the problem type
  (from large shipyard production with unary parts on hundreds of sheets, to small production
  with high quantities of small parts)
- Optimizes to minimize total surface used, with offcut kept on last sheet
- Nesting optimized to the left, offcut on right, using **vertical front line**
- Rotation of parts around **any angle**
- Management of **prohibited part positions** and nesting priorities
- **Compaction** of existing nestings
- Claims **5% savings over manual nesting** and **0.5% over competing software**
- Cloud computing option for virtually unlimited computation time

### Features

- **Full automatic mode** with interactive fallback for complex situations
- **Common-line cutting** support
- **Multi-head cutting** support
- **Protection areas** (keep-out zones)
- **Unloading constraints** awareness
- **Coil nesting** (continuous material, not just sheets)
- Stock and manufacturing order management
- Quotation module
- Letter and drawing cutting
- Sheet metal part unfolding
- NC code generation for any machine type

### Product Variants

- **PowerNest Lib**: C/C++/C# library (DLL) for integration into third-party software
- **PowerNest WS**: RESTful API web service (cloud-based nesting)
- **PowerNest Square**: rectangular/linear cutting optimization
- **PowerNest Cube**: 3D nesting of parallelepipedic shapes
- **AlmaCAM/act/cut**: complete CAM software for end users

### Input/Output Formats

- Imports from **any CAD software** (DXF and standard formats)
- Integration with ERP/production management systems
- NC code generation for all machine types

### Strengths

- **45 years of dedicated nesting algorithm R&D** -- deepest expertise in the market
- **Dual delivery model** (library API + end-user software) proves algorithm quality
- **Cloud-based nesting** with unlimited computation time
- **Algorithm auto-selection** based on problem characteristics
- **RESTful API** available (PowerNest WS) -- modern integration approach
- 0.5% better than competitors on average (verified by cloud demo)
- Employee-owned cooperative -- aligned incentives for quality

### Weaknesses

- Less well-known brand than SigmaNEST or ProNest in North America
- Primarily European market presence
- PowerNest library licensing may be expensive for small integrators
- Documentation and marketing in French primarily

### Pricing Model

- Quote-based for both library licensing and end-user software
- PowerNest cloud demo available for free benchmarking
- Not publicly listed

### Relevance to Thetta

- **PowerNest's dual delivery model** (library + web service) is exactly the architecture
  pattern Thetta should consider
- The **automatic algorithm selection** based on problem characteristics is sophisticated --
  Thetta could implement strategy selection heuristics
- **Vertical front line** optimization approach is a documented technique worth studying
- **Cloud-based nesting with unlimited computation** is a compelling product model
- The RESTful API approach (PowerNest WS) validates Thetta's potential as a service
- Their 0.5% advantage claim gives a benchmark for what "best-in-class" means

---

## 7. Lantek Expert

### Overview

Lantek Expert is an advanced CAD/CAM nesting software system with 30+ years of experience,
designed for automating CNC programming of sheet metal machines. It supports cutting (laser,
plasma, oxyfuel, waterjet) and punching machines. Lantek calls their technology approach
"Thinkingmetal Technology."

### Nesting Algorithm

- **Thinkingmetal Technology** -- proprietary advanced nesting algorithms
- Combination of **automatic, semi-automatic, and manual nesting**
- Automatic nesting optimizes part arrangement for maximum utilization of parts and remnants
- Automatic detection and prioritization of **remnants in warehouse** before using new sheets
- Configurable tables by **material/thickness** for separation distances, lead-ins/lead-outs
- New automatic **micro-joints placement** algorithm
- **Machining by angle**: different machining strategies per nesting angle, stored in database

### Features

- **Intelligent Collision Avoidance**: understands cutting path and head travel speed, auto-applies
  tabs to avoid part flip-ups
- **Remnant management**: auto-detects remnants and prioritizes their use
- **A2N 3D Importer**: unfolds 3D sheet metal parts with bend deduction
- **Standard and 3D Bevel** capabilities (multiple bevel types per part)
- **Machining by Angle**: stores angle-specific machining strategies in database
- **Multi-machine reassignment**: quickly move nests between machines
- Full **ERP integration** (Lantek Integra)

### Specialized Modules

- **Lantek Expert Cut**: laser, plasma, oxyfuel, waterjet
- **Lantek Expert Punch**: punching machines
- **Lantek Expert Duct**: HVAC/duct figures (DIN/SMACNA parametric library)
- **Lantek Expert Quattro**: shear machines (rectangular parts, remnant reuse)

### Input/Output Formats

- **DXF, DWG, IGES, DSTV** and other major CAD formats
- **3D CAD import** with automatic unfolding (most common formats)
- NC code generation for all supported machines

### Strengths

- **Intelligent collision avoidance** that understands physics of cutting
- **Thinkingmetal Technology** with machining-by-angle awareness
- Broad machine compatibility (Amada, TRUMPF, Mazak, Prima Power, etc.)
- Strong **remnant management** with automatic warehouse detection
- Full vertical integration (CAD/CAM/MES/ERP via Lantek Integra)
- Good balance of automation and manual control

### Weaknesses

- Quote-based pricing (not transparent)
- Complex product line may be confusing (Cut, Punch, Duct, Quattro)
- Learning curve for full feature utilization
- Some users report UI could be more modern

### Pricing Model

- **Quote-based** -- contact Lantek or authorized resellers
- **Floating license** option for network installations
- Modular purchasing -- buy more advanced modules as needs grow
- One reviewer noted it is "cheap to license" (relative to competitors)

### Relevance to Thetta

- **Machining by angle** concept (storing angle-specific strategies) is unique and powerful
- **Intelligent collision avoidance** shows the value of physics-aware nesting
- **Remnant prioritization** is a production feature that adds real value
- The modular architecture (Cut, Punch, Duct, Quattro) shows how to segment a nesting product

---

## 8. NestLib (HCL Technologies / Geometric)

### Overview

NestLib is a **nesting algorithm library** (not end-user software) developed by HCL Technologies
(formerly Geometric Software Solutions). Licensed by **125+ OEMs worldwide**, it is recognized as
one of the fastest and most efficient nesting libraries in the industry. It powers many other
commercial nesting products behind the scenes.

### Nesting Algorithm

- **True-shape nesting** algorithms (core capability)
- **Rectangular nesting** with specialized 2D bin-packing algorithms
- **Adaptive nesting engine** that can influence results for better outcomes
- **Multi-core parallelization**: up to 40% faster on dual-core, 70% faster on quad-core
- **Strip nesting**: packs copies of a single part in a dense manner without sheet input
- **Shear nesting**: for saw parts requiring end-to-end cutting
- **Common cut/flame path** sharing between adjacent parts

### Modules

| Module | Description |
|--------|-------------|
| **Base Module** | Core automated nesting algorithms |
| **Adaptive Nesting** | Dynamic nesting engine |
| **Automatic Pairing & Clusters** | Automated part grouping |
| **Common Flame Cutting** | Shared cut paths for sheet metal |
| **Common Punch** | Shared punch operations |
| **Cutting Sequence Generation** | Optimal cut ordering |
| **Grid Fit** | Grid-based arrangement |
| **Inventory Forecasting** | Stock prediction |
| **Leather Nesting** | Irregular material with defects |
| **Master Plates** | Template-based plate management |
| **Multiple Torch** | Multi-torch cutting |
| **Optimizer** | Advanced optimization |
| **Rectangular Nesting** | Specialized rectangular bin-packing |
| **Remnant Plate Generation** | Leftover material tracking |
| **Shear Cutting** | Shear machine integration |
| **Speed Nesting** | High-speed nesting |
| **Strip Nesting** | Dense single-part packing |
| **Inclined Guillotine Cut** | Non-rectangular guillotine cuts |
| **Tube Nesting** | Cutting tubes from larger stock |
| **Overlap Detection** | Layout error detection |

### Input/Output Formats

- API-based (no direct file I/O -- integrating application handles this)
- Available as **static library, DLL, COM DLL**
- Supports **Java and .NET framework**
- Platforms: Windows (32/64-bit), Sun Solaris, Linux, Mac OS (on demand)

### Strengths

- **Industry standard** nesting engine used by 125+ OEMs
- **Most comprehensive module set** of any nesting library
- **Multi-core support** for performance
- **Cross-platform** (Windows, Linux, Solaris, Mac)
- **Multiple API types** (C/C++, COM, .NET, Java)
- Proven at scale across leather, wood, metal, glass, composites, packaging
- Overlap detection for quality assurance

### Weaknesses

- **Not end-user software** -- requires integration into a host application
- **OEM licensing model** -- likely expensive for small developers
- Closed-source / proprietary
- Limited public documentation of algorithm internals
- No web/cloud API (traditional library only)

### Pricing Model

- **OEM licensing** -- quote-based, likely per-seat or per-deployment royalties
- Not available for individual purchase
- Contact HCL Technologies for licensing terms

### Relevance to Thetta

- NestLib's **modular architecture** is the gold standard for nesting libraries
- The **module list** provides a comprehensive feature roadmap for Thetta
- **Multi-core parallelization** is essential for competitive performance
- **Strip nesting** (single-part dense packing without sheet) is a useful primitive
- **Adaptive nesting** concept (influencing results dynamically) is worth studying
- The fact that 125+ OEMs license NestLib validates the market for nesting libraries
- Thetta could position itself as an open-source alternative to NestLib

---

## 9. Astra R-Nesting / Astra S-Nesting (Technos)

### Overview

Astra is a nesting software suite by Technos LLC (based in Mykolayiv, Ukraine). It comes in
two variants: **Astra R-Nesting** for rectangular nesting, and **Astra S-Nesting** for true-shape
nesting. Used for cutting particle board, metal, glass, plastics, fabric, leather, wood, and
stone.

Note: The user-requested "Nesting Genius" does not appear to be a distinct product. No software
by that exact name was found in extensive searching. It may be a feature name, a colloquial
reference, or confused with another product.

### Nesting Algorithm

- **Astra R-Nesting**: Specialized rectangular 2D nesting with a "unique high-speed algorithm"
  - Achieves **95-97% coverage** on average
  - Generates layouts for 2,000 parts in **<10 seconds**
  - Takes into account technological and organizational parameters of production
- **Astra S-Nesting**: True-shape nesting optimization
  - Imports parts from DXF files
  - Automatic sorting into nesting groups by thickness/material
  - Quick automatic nesting with high material usage ratio

### Features

- **Order creation** with standard assemblies, manual entry, or import
- **Customizable parameters**: width of cut, edging, through cuts, nesting type, minimum saw spacing
- **Manual editing** of cutting layouts with specialized functions
- **Offcut management**: automatic and dialog-mode calculation and saving of offcuts
- **Reports and labels** printing
- **Export to NC machines** (ISO G-codes, ESSI, DXF)
- **Inventory monitoring** and order merging
- **Postprocessors** for specific cutting machines (Mayer, Felder)
- **Configurable import** from Excel and CADWORK

### Input/Output Formats

- **DXF import** for parts
- **Excel import** (configurable)
- **CADWORK import**
- **Export**: ISO G-codes, ESSI, DXF for CNC machines

### Strengths

- **Extremely fast** rectangular nesting (2,000 parts in <10 seconds)
- **High coverage** (95-97% for rectangular nesting)
- **Affordable**: Astra S-Nesting at $999 (one-time purchase)
- Good for panel-based industries (furniture, particle board)
- Practical features (offcut management, inventory, Excel import)

### Weaknesses

- **Niche product** with limited market visibility
- Limited CAD format support (primarily DXF)
- Smaller development team (Ukrainian company)
- Limited documentation in English
- No cloud or API offering
- True-shape nesting capabilities less proven than major competitors

### Pricing Model

- **Astra S-Nesting**: $999 one-time purchase
- **Astra R-Nesting**: Available for purchase (price not found publicly)
- Free trial versions available

### Relevance to Thetta

- The **95-97% coverage** claim for rectangular nesting provides a benchmark
- **<10 second** performance for 2,000 parts shows what's achievable with optimized algorithms
- Excel/CADWORK import shows practical integration needs for real workshops
- The R-Nesting / S-Nesting split mirrors the rectangular vs. true-shape algorithm divide

---

## 10. JetCAM

### Overview

JetCAM develops CAM/nesting and IoT Industry 4.0 automation software for sheet metal and
composite manufacturing. With **500+ postprocessors** for CNC machines, it is one of the most
broadly compatible nesting solutions. Notable customers include ABB, BAE, Bell Helicopter,
Bombardier, Embraer, Electrolux, GE, Hitachi, and Volvo.

### Nesting Algorithm

JetCAM offers a **tiered progression of nesting algorithms**, each building on the previous:

1. **Rectangular Nesting**: Two versions -- standard single-pass + HP-RAN (time-based, tries
   different patterns over a set time period)
2. **Free Form Automatic Nesting (FFAN)** (1993): True-shape nesting delivering near-instant results
3. **High Performance Nesting (HPN)**: Time-based algorithm -- user sets maximum time, algorithm
   continues trying different patterns for greater efficiency
4. **Ultra Performance Nesting (UPN)**: Extremely aggressive algorithm for large numbers of
   dissimilar parts; creates multiple sheets simultaneously considering all available stock
5. **Genetic algorithms**: Latest module using genetic algorithms for "unparalleled packing
   optimization and processing speed"

### Features

- **500+ CNC postprocessors** (punch, laser, plasma, combination, oxyfuel, waterjet, router, knife)
- **Common-line cutting** for parts with shared edges
- **Part-in-part nesting** with automatic spreading for heat avoidance
- **Progressive nesting**: groups parts by assembly for faster kitting
- **Hybrid progressive nesting**: configurable maximum distance from group for efficiency balance
- **Remote Control Processing (RCP)**: turns JetCAM into a "black box" for ERP-driven automation
- **JETCut automatic fly cutting** and heat avoidance
- **Automatic skeleton destruction**
- **"Prefer Less Layouts"** option (toggleable: none/little/normal/strong/aggressive)

### Input/Output Formats

- Standard CAD format import
- NC code generation via 500+ postprocessors
- Integration with external ERP systems via RCP

### Strengths

- **500+ postprocessors** -- broadest machine compatibility available
- **Tiered algorithm selection** from fast to ultra-optimized
- **Genetic algorithm** module for cutting-edge optimization
- **10%+ more efficient** than benchmarked competitors
- **600% improvement** in CADCAM and nesting time reported
- **Hybrid progressive nesting** balances efficiency vs. practical sorting
- Strong aerospace and defense customer base

### Weaknesses

- Quote-based pricing (not transparent)
- Complex product tiers may be confusing
- UPN and genetic algorithm modules are premium add-ons
- Less focus on integrated production management vs. SigmaNEST/ProNest
- Primarily Windows-based

### Pricing Model

- **Perpetual purchase** (one-time license) or **subscription/rental**
- No pay-per-use model (avoids charging for exploratory nests)
- Subscription can be stopped and converted toward purchase
- License replacement: EUR 160 (standalone) / EUR 320 (floating)
- Multiple tiers: Expert Lite, Expert 1, Expert Premium, Premium Connected
- Contact reseller for specific pricing

### Relevance to Thetta

- **Tiered algorithm progression** (rectangular -> true-shape -> time-based -> genetic) provides
  a clear feature roadmap
- **Genetic algorithms** for nesting optimization are proven at industrial scale
- **Time-based nesting** (keep trying until time runs out) is a powerful UX concept
- **Hybrid progressive nesting** (balancing efficiency vs. practicality) is unique
- The "Prefer Less Layouts" concept shows users want control over sheet count vs. efficiency
- **600% time improvement** claims give a benchmark for what good tooling achieves

---

## 11. Commercial vs. Open-Source Nesting

### Key Open-Source Tools

| Tool | Algorithm | Strengths | Limitations |
|------|-----------|-----------|-------------|
| **Deepnest** | Genetic algorithm + NFP | Free, cross-platform, common-line merging | No multi-sheet selection, no editing, no ERP integration |
| **SVGnest** | Genetic algorithm + NFP | Free, web-based, part-in-part, OS-independent | Browser-only, limited to SVG, can produce long slivers of waste |
| **libnest2d** | NFP-based C++ library | Embeddable, used in PrusaSlicer | Limited features, focused on 3D printing bed nesting |

### What Makes Commercial Nesting Better

1. **Algorithm Maturity**: Commercial tools have 20-45 years of algorithm refinement. They
   consistently achieve 0.5-5% better material utilization than open-source alternatives,
   which translates to significant cost savings at industrial scale.

2. **Problem-Specific Algorithm Selection**: Tools like PowerNest automatically select the
   best algorithm for the specific problem type. Open-source tools typically offer one algorithm.

3. **Production Awareness**: Commercial tools understand cutting physics -- kerf width, heat
   distortion, lead-in/lead-out placement, collision avoidance, tip-up prevention. Open-source
   tools treat nesting as a pure geometry problem.

4. **Multi-Constraint Optimization**: Commercial tools simultaneously optimize for material
   usage, cutting time, tool wear, grain direction, part quality, and assembly grouping.
   Open-source tools primarily optimize for material usage alone.

### Features in Commercial Tools Missing from Open Source

| Feature | Commercial Examples | Open-Source Status |
|---------|-------------------|-------------------|
| **Remnant/inventory management** | SigmaNEST, Lantek, ProNest | Not available |
| **Common-line cutting (dissimilar parts)** | ProNest Advanced CLC | Basic CLC only (Deepnest) |
| **Collision avoidance** | ProNest, Lantek | Not available |
| **ERP/MRP integration** | All major tools | Not available |
| **3D CAD unfolding** | SigmaNEST, Lantek A2N | Not available |
| **Multi-machine postprocessing** | JetCAM (500+ posts) | Not available |
| **Grain direction control** | NestFab, SigmaNEST, OptiNest | Not available |
| **Part-in-part nesting** | SigmaNEST, SVGnest | SVGnest only (limited) |
| **Time-based optimization** | JetCAM HPN, NestFab UP | Not available |
| **Adaptive/learning algorithms** | SigmaNEST, NestLib | Not available |
| **Cut sequence optimization** | ProNest, JetCAM, NestLib | Not available |
| **Bridge/chain cutting** | ProNest | Not available |
| **Multi-sheet stock optimization** | JetCAM UPN, PowerNest | Not available |
| **Heat distortion management** | JetCAM, ProNest | Not available |
| **Quoting and costing** | SigmaNEST, ProNest | Not available |
| **Real-time machine monitoring** | ProNest Production Manager | Not available |
| **Cloud-based nesting API** | PowerNest WS | Not available |

### The Real Gap

The most significant gap between commercial and open-source nesting is not in the core packing
algorithm (where open-source tools like SVGnest/Deepnest are surprisingly competitive), but in:

1. **Production workflow integration** -- connecting nesting to the entire manufacturing process
2. **Cutting-technology awareness** -- understanding how the physical cutting process constrains
   the nesting solution
3. **Multi-constraint optimization** -- balancing competing objectives simultaneously
4. **Computation time investment** -- commercial tools allow extended computation for better results
5. **Algorithm selection intelligence** -- choosing the right approach for the problem at hand

---

## 12. Key Takeaways for Thetta

### Algorithm Insights

1. **NFP (No-Fit Polygon) is the foundation** of all serious true-shape nesting. Every major
   commercial tool uses it (explicitly or implicitly). Thetta must have robust NFP computation.

2. **Time-based optimization** is a proven commercial pattern. Let users trade computation time
   for better results (JetCAM, NestFab, PowerNest, TRUMPF).

3. **Genetic algorithms** are used at industrial scale (JetCAM, SVGnest/Deepnest). They work.

4. **Algorithm auto-selection** based on problem characteristics (PowerNest) is a differentiating
   feature worth pursuing.

5. **Multi-core parallelization** is essential. NestLib reports 40-70% speedup on multi-core.

### Feature Priorities (Based on Commercial Tool Analysis)

**Must-have** (all commercial tools have these):
- True-shape nesting with NFP
- Part rotation (configurable angles including free rotation)
- Part-in-part nesting
- Grain direction constraints
- Kerf-width-aware spacing
- Multiple sheet sizes/stocks
- Remnant/offcut tracking

**High-value** (differentiating features):
- Common-line cutting (shared edges)
- Time-based optimization (keep improving until time limit)
- Tilt tolerance for grain (NestFab's +/- degree concept)
- Strip nesting for single-part optimization
- Configurable algorithm parameters (OptiNest model)

**Advanced** (production-grade features):
- Cut sequence optimization
- Collision avoidance
- Bridge/chain cutting
- Multi-head machine support
- ERP integration / API

### Architecture Insights

1. **Library-first architecture** is validated by NestLib (125+ OEM customers) and PowerNest
   (library + web service + end-user software). Thetta should be a library first.

2. **Modular design** is universal. Every commercial tool uses modules for optional features.
   Thetta's core should be minimal with composable extensions.

3. **Web service / API delivery** is the modern direction (PowerNest WS RESTful API). Thetta
   should support this deployment model.

4. **Cross-platform support** matters. NestLib supports Windows/Linux/Solaris/Mac. Rust gives
   Thetta this advantage natively.

### Competitive Positioning

Thetta can differentiate from commercial tools by:
- Being **open-source** (no vendor lock-in, community contributions)
- Offering a **modern API** (REST/gRPC vs. COM/DLL)
- Supporting **Rust/WASM** for both native and browser deployment
- Providing **transparent algorithms** (reproducible results)
- Focusing on the **core nesting problem** without manufacturing-specific bloat

The realistic benchmark for Thetta's nesting quality:
- **Open-source baseline** (Deepnest/SVGnest): match or exceed
- **Commercial target**: within 1-2% of SigmaNEST/PowerNest material utilization
- **Speed target**: competitive with NestLib on multi-core (sub-second for typical problems)

---

## Sources

### SigmaNEST
- [SigmaNEST Official](https://www.sigmanest.com/en/sigmanest)
- [SigmaNEST Reviews - SelectHub](https://www.selecthub.com/p/manufacturing-software/sigmanest/)
- [SigmaNEST Reviews - G2](https://www.g2.com/products/sigmanest/reviews)
- [SigmaNEST Packages](https://sigmanest.ib-caddy.com/packages.php)
- [SigmaNEST CAD Integration](https://www.sigmanest.com/en/cad-integration)

### ProNest
- [ProNest Official](https://www.hypertherm.com/hypertherm/pronest/pronest-cadcam-nesting-software/)
- [ProNest Optional Modules](https://www.hypertherm.com/products/software/pronest-optional-modules/)
- [ProNest 2025 v16.0 Release](https://www.hypertherm.com/resources/more-resources/news/pronest-2025-v16-cadcam-nesting-software/)
- [ProNest LT Subscription Plans](https://www.hypertherm.com/products/software/pronest-lt-subscription-plans/)
- [Common Line Cutting with ProNest](https://www.hypertherm.com/resources/more-resources/blogs/common-line-cutting/)

### TRUMPF TruTops
- [TruTops Boost Official](https://www.trumpf.com/en_INT/products/software/programming-software/trutops-boost/)
- [TruTops Boost New Release](https://www.trumpf.com/en_US/products/software/programming-software/trutops-boost/trutops-boost-new-release/)
- [TRUMPF Software Licensing](https://www.trumpf.com/en_US/products/software/software-licensing/)

### NestFab
- [NestFab Features](https://www.nestfab.com/features)
- [NestFab Overview](https://www.nestfab.com/)
- [NestFab Pricing](https://www.nestfab.com/pricing)
- [NestFab V3](https://www.nestfab.com/v3)

### OptiNest
- [OptiNest Official](https://boole.eu/test/optinest.php)
- [OptiNest Boole EU](http://boole.eu/optinest_eng.php)
- [OptiNest User Manual (PDF)](https://boole.eu/pdf/Manuel_ON_eng.pdf)
- [OptiNest - Wood Designer](https://wooddesigner.org/optinest-nesting-software/)

### Alma / PowerNest
- [AlmaCAM Official](https://almacam.com/)
- [PowerNest Library](https://powernestlib.com/)
- [PowerNest True Shape Nesting](https://powernestlib.com/true-shape-nesting-library/)
- [PowerNest WS Documentation](https://doc.nesting.almacam.com/)
- [PowerNest Cloud Demo](https://powernestlib.com/automatic-nesting-demonstrator-in-the-cloud/)

### Lantek Expert
- [Lantek Expert Official](https://www.lantek.com/uk/Lantek-Expert-Nesting-Software)
- [Lantek Expert Cut](https://www.lantek.com/us/cad-cam-nesting-software-oxycut-plasma-laser-waterjet)
- [Lantek v45 Release](https://www.modernmetals.com/lantek-launches-v-45-of-its-lantek-suite-software.html)
- [Lantek Expert Reviews - G2](https://www.g2.com/products/lantek-expert/reviews)

### NestLib
- [NestLib Official](https://nestlib.geometricglobal.com/)
- [NestLib Modules](https://nestlib.geometricglobal.com/modules/)
- [NestLib FAQs](https://nestlib.geometricglobal.com/support/faqs/)
- [NestLib v19 Release](https://www.prweb.com/releases/2006/11/prweb468032.htm)

### Astra Nesting
- [Astra R-Nesting](https://www.astranest.com/)
- [Astra S-Nesting](https://www.nesting-soft.com/)
- [Technos Official](http://www.techno-sys.com/)

### JetCAM
- [JetCAM Official](https://www.jetcam.net/)
- [JetCAM Expert](https://www.jetcam.net/expert.htm)
- [JetCAM Nesting Modules](https://www.jetcam.net/nesting-modules.htm)
- [JetCAM Ultra Performance Nesting](https://files.jetcam.net/ultra-performance-nesting/)
- [JetCAM Purchase vs Subscription](https://pages.jetcam.net/blog/purchase-or-subscription-which-model-is-right-for-you)

### General / Comparison
- [What is Nesting Software - Hypertherm](https://www.hypertherm.com/solutions/technology/cam-software/nesting-software/)
- [NFP in Shape Nesting - Nestable](https://www.nestable.xyz/blog/introduction-nfp-no-fit-polygon/)
- [Best Nesting Software - CNCSourced](https://www.cncsourced.com/rankings/best-nesting-software-for-laser-cutters/)
- [What is Nesting Software - CADTalk](https://cadtalk.com/what-is-nesting-software/)
- [Deepnest.io](https://deepnest.io/)
- [SVGnest](https://svgnest.com/)
