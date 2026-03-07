# Research Papers Analysis: 2D Irregular Nesting and Packing

This document provides a detailed analysis of 10 research papers relevant to the Thetta project, covering algorithms for 2D irregular cutting, packing, and nesting.

---

## Paper 1: Complete and Robust No-Fit Polygon Generation for the Irregular Stock Cutting Problem

**Title and Authors:** "Complete and robust no-fit polygon generation for the irregular stock cutting problem" by E.K. Burke, R.S.R. Hellier, G. Kendall, and G. Whitwell.

**Year and Publication Venue:** 2007, European Journal of Operational Research, Vol. 179, pp. 27-49.

**Problem Addressed:** Generation of No-Fit Polygons (NFPs) for pairs of irregular 2D shapes in the context of stock cutting. The NFP is a geometric construct that enables fast overlap detection between two polygons: if the reference point of one polygon lies outside the NFP boundary, the two shapes do not overlap.

**Method/Algorithm:** The paper introduces a robust orbital sliding method for NFP generation:
1. One polygon (the "orbiting" piece) is placed in contact with the other ("stationary" piece).
2. The orbiting piece slides around the stationary piece, maintaining contact at all times, tracing a path with its reference point.
3. The traced path forms the outer boundary of the NFP.
4. To handle concavities and holes, the algorithm identifies feasible starting positions inside holes of the stationary piece where the orbiting piece can be placed.
5. The algorithm uses edge-to-edge touching detection and manages special degenerate cases (jigsaw-type interlocking, perfect fits, sliding along parallel edges).
6. Only two geometric stages are required: (a) identifying start positions, and (b) performing the orbital slide.

**Key Innovations:**
- A complete NFP generation algorithm that handles all known degenerate cases (holes, interlocking concavities, jigsaw pieces, perfect fits) without special-case handling.
- Robust handling of numerical precision issues that plague other approaches.
- Simple two-stage approach that is easy to understand and implement.

**Results:** NFP generation times are reported for 32 irregular packing benchmark problems from the literature, including real-world datasets. The algorithm successfully generates NFPs for all tested cases, including those with known difficult degeneracies.

**Strengths:**
- Handles all degenerate cases without ad-hoc fixes.
- Conceptually simple (only two geometric stages).
- Provides timing benchmarks on a comprehensive set of 32 problems.
- Has become a foundational reference for subsequent NFP work.

**Weaknesses:**
- The orbital approach can be slow for highly complex pieces with many vertices.
- Numerical precision issues, while addressed, still require careful epsilon handling.
- Only supports polygonal representations (no curves/arcs).

**Relevance to Thetta Project:** NFP generation is the core geometric primitive needed for any nesting engine. This paper provides the foundational algorithm for computing pairwise overlap constraints. In a STEP-to-nest-to-STEP pipeline, after extracting 2D profiles from STEP faces, the NFP between every pair of pieces (and between each piece and the rectangular bin boundary, i.e., the Inner-Fit Polygon) must be computed. This paper's orbital method is the most widely adopted approach for this purpose.

---

## Paper 2: Robust NFP Generation for Nesting Problems

**Title and Authors:** "Robust NFP generation for Nesting problems" by Pedro Rocha.

**Year and Publication Venue:** 2019, arXiv preprint (arXiv:1903.11139v1).

**Problem Addressed:** Generating complete No-Fit Polygons for nesting, with a focus on reducing numerical precision errors, simplifying the resulting NFP (fewer vertices/edges), and correctly identifying all degenerate cases including holes, perfect fits, and perfect sliding positions.

**Method/Algorithm:**
1. **Convex Decomposition:** Both pieces A and B are decomposed into convex sub-polygons using any covering or partition algorithm.
2. **Convex NFP Generation:** Compute convex NFPs between all pairs of convex sub-polygons (one from A, one from B) using slope diagrams.
3. **Intersection Detection:** Compute all intersection points between pairs of convex NFP components.
4. **Edge Midpoint Insertion:** Insert midpoint vertices on every edge of every convex NFP component. This is critical for enabling correct containment tests.
5. **Graph Construction:** Convert all convex NFP components into a directed graph with shared vertices.
6. **Containment-Based Pruning:** Test every vertex for containment inside each convex NFP component. Remove all vertices (and their edges) that lie strictly inside any component. This leaves only the boundary vertices of the full NFP.
7. **Outline Extraction:** Use an angle-based extraction algorithm to identify the outer boundary, holes, perfect sliding positions, and perfect fit positions from the remaining graph.

**Key Innovations:**
- The midpoint insertion trick enables robust containment testing without complex geometric algorithms.
- The approach naturally handles degenerate cases (holes, perfect fits, perfect slides) through graph-based outline extraction rather than special-case handling.
- The resulting NFP has a minimal number of geometric components.
- The algorithm can be extended to boolean operations (union, intersection, XOR, NOT) on polygons, and can even work in 3D.

**Results:** Successfully generates correct NFPs for all known problem cases from Burke et al. (2007), including interlocking concavities, multiple holes, perfect fits, and jigsaw pieces. Validated against literature benchmark cases.

**Strengths:**
- Simple and elegant algorithm based on convex decomposition and graph pruning.
- Produces minimal NFPs (fewer vertices and edges).
- Naturally extensible to boolean operations and 3D.
- Robust against numerical precision problems.

**Weaknesses:**
- Computational cost of the containment test (every vertex against every convex NFP component) could be expensive for highly complex pieces.
- Relies on quality of convex decomposition -- more sub-polygons means more pairwise NFPs.
- No performance timing comparisons against other methods.

**Relevance to Thetta Project:** This algorithm offers a modern, robust alternative to the Burke (2007) orbital approach for NFP computation. Its extensibility to boolean operations is valuable for computing union shapes (merged pieces). For the Thetta pipeline, the convex decomposition approach may be advantageous since STEP geometry often involves convex faces naturally. The 3D extension hints at future capability for 3D nesting.

---

## Paper 3: Two-Dimensional Irregular Packing Problems: A Review

**Title and Authors:** "Two-dimensional irregular packing problems: A review" by Baosu Guo, Yu Zhang, Jingwen Hu, Jinrui Li, Fenghe Wu, Qingjin Peng, and Quan Zhang.

**Year and Publication Venue:** 2022, Frontiers in Mechanical Engineering, Vol. 8, Article 966691.

**Problem Addressed:** Comprehensive survey of the entire field of 2D irregular packing/nesting problems, covering geometric representations, solution approaches, benchmark problems, and applications.

**Method/Algorithm:** This is a review paper. It categorizes the field along several dimensions:
1. **Problem types:** Strip packing (minimize length), bin packing (minimize number of bins), open-dimension packing, constrained packing.
2. **Geometric representations:** Direct trigonometry, raster/pixel methods, no-fit polygons (NFPs), phi-functions, circle covering.
3. **Solution approaches:**
   - Constructive heuristics (bottom-left, bottom-left-fill, best-fit).
   - Metaheuristics (genetic algorithms, simulated annealing, tabu search, cuckoo search).
   - Exact methods (MIP models, constraint programming).
   - Hybrid methods combining placement heuristics with global search.
4. **Applications:** Textiles/garments, sheet metal cutting, leather, glass, shipbuilding, furniture.

**Key Innovations (Identified in the Review):**
- The Guided Cuckoo Search (Elkeran, 2013) is identified as the state-of-the-art heuristic method.
- NFP-based geometric representations dominate practical implementations.
- Hybrid approaches combining local search with global metaheuristics show the best performance.
- Machine learning approaches are emerging but not yet competitive.

**Results:** Comprehensive benchmark comparison tables and classification of methods by problem type and solution quality.

**Strengths:**
- Thorough coverage of the entire field up to 2022.
- Clear taxonomy of problem types and solution methods.
- Identifies key open challenges and research gaps.
- Useful bibliography for finding specific algorithms.

**Weaknesses:**
- As a review, it does not introduce new algorithms.
- Some coverage is superficial, especially on emerging methods.
- Does not include post-2022 breakthroughs.

**Relevance to Thetta Project:** Essential reading for understanding the landscape of nesting algorithms. Key takeaways for Thetta: (1) NFP is the dominant geometric representation for practical systems; (2) a two-level approach (placement heuristic + metaheuristic search) is the standard architecture; (3) bottom-left-fill is the most common placement heuristic; (4) for fixed rectangular bins, the bin packing variant is the relevant problem formulation.

---

## Paper 4: 2D Irregular Optimization Nesting Method Based on Adaptive Probabilistic Genetic Simulated Annealing Algorithm

**Title and Authors:** "2D Irregular Optimization Nesting Method based on Adaptive Probabilistic Genetic Simulated Annealing Algorithm" by Xujia Qin, Linjiang Jin, and Hongbo Zheng.

**Year and Publication Venue:** 2020, Computer-Aided Design and Applications (CAD&A), Vol. 18, pp. 242-257.

**Problem Addressed:** 2D irregular nesting on irregular motherboards (not just rectangular sheets), optimizing material utilization. Both pieces and motherboards can be irregular polygons.

**Method/Algorithm:**
1. **NFP Computation:** Use the no-fit polygon algorithm to compute feasible placement regions.
2. **Central Expansion Strategy:** Generate an initial nesting layout by expanding from the center of the motherboard outward. This fills the central area first.
3. **Genetic Algorithm (GA):** Encode nesting solutions as chromosomes representing the sequence and rotation angle of pieces.
   - Selection: Roulette wheel selection.
   - Crossover: Order crossover adapted for nesting.
   - Mutation: Swap and rotation mutation operators.
4. **Simulated Annealing (SA):** Integrated with GA to escape local optima. After GA generates candidate solutions, SA applies local perturbations and accepts worse solutions with a temperature-dependent probability.
5. **Adaptive Probability:** The crossover and mutation probabilities are dynamically adjusted based on the fitness diversity of the current population, preventing premature convergence.

**Key Innovations:**
- Hybrid GA-SA algorithm with adaptive probability control.
- Central expansion strategy for initial placement (different from typical bottom-left).
- Support for irregular motherboards (not just rectangular strips).
- Adaptive crossover/mutation rates based on population fitness variance.

**Results:** Tested on garment nesting problems. The algorithm achieves utilization rates of approximately 85-92% on various test cases, improving on standalone GA or SA approaches. Convergence is faster due to the adaptive probability mechanism.

**Strengths:**
- Handles irregular motherboards, not just rectangular bins.
- Adaptive probability prevents premature convergence.
- Combination of global (GA) and local (SA) search.
- Practical focus on garment industry applications.

**Weaknesses:**
- Limited benchmark comparison -- does not compare against state-of-the-art methods like Guided Cuckoo Search.
- The central expansion strategy may not be optimal for rectangular bins.
- Relatively small test instances.
- No open-source implementation.

**Relevance to Thetta Project:** The adaptive GA-SA framework is a reasonable metaheuristic approach for the combinatorial search component of nesting. However, since Thetta uses fixed rectangular bins (not irregular motherboards), the central expansion strategy is less relevant. The adaptive probability mechanism could be borrowed for Thetta's search layer, but the Sparrow paper (Paper 8) offers a more competitive and reproducible alternative.

---

## Paper 5: A New Mixed-Integer Programming Model for Irregular Strip Packing Based on Vertical Slices

**Title and Authors:** "A new mixed-integer programming model for irregular strip packing based on vertical slices with a reproducible survey" by Juan J. Lastra-Diaz and M. Teresa Ortuno.

**Year and Publication Venue:** 2023 (arXiv preprint 2022, submitted to journal), arXiv:2206.00032v1.

**Problem Addressed:** Exact (optimal) solutions to the irregular strip packing problem: place non-convex polygons on a fixed-width, unbounded-length rectangular strip to minimize strip length.

**Method/Algorithm:**
1. **Convex Decomposition:** Each non-convex piece is decomposed into convex parts using Greene's algorithm (from CGAL).
2. **Convex NFP Computation:** For each pair of convex parts from different pieces, compute their convex NFP using Minkowski sums.
3. **Vertical Slice Decomposition:** The feasible region defined by each convex NFP is decomposed into disjoint convex sub-regions using vertical slices. Each sub-region is assigned a mutually exclusive binary variable.
4. **MIP Formulation (NFP-CM-VS):** The non-overlapping constraints are encoded as linear inequalities activated by binary variables. The model includes:
   - Non-overlapping constraints based on convex NFP edges.
   - Vertical slice constraints that break symmetries in the feasible solution space.
   - Valid inequalities for identical pieces (variable eliminations).
   - Feasibility cuts among triplets of pieces.
   - Clique-based cuts to compress sets of mutual-exclusion constraints.
5. **NFP-CM-VS2 variant:** A more compact formulation that removes big-M terms for vertical slice constraints by factorizing x-axis constraints.
6. **Solver:** Gurobi 9.5 (Branch & Cut).

**Key Innovations:**
- Novel disjoint convex decomposition of feasible placement regions into vertical slices.
- New family of valid inequalities among three pieces (feasibility cuts using interval arithmetic).
- Symmetry-breaking for identical pieces via variable elimination.
- Clique-based encoding of non-overlapping constraints.
- First reproducible benchmark with complete open-source Java implementation (RAMNEST library).

**Results:**
- On 35 small instances (3-16 pieces): NFP-CM-VS solves the most instances to optimality, outperforming NFP-CM and Improved NFP-CM models.
- On 11 large instances (16-64 pieces): NFP-CM-VS obtains lower terminating gaps than Improved NFP-CM and finds feasible solutions in more cases.
- The three-piece feasibility cuts significantly improve performance.
- NFP-CM-VS2 provides comparable performance with a more compact formulation.
- All models struggle with instances > 20 pieces or complex geometries.

**Strengths:**
- Provably optimal solutions (when solvable).
- Reproducible: open-source Java code and dataset provided.
- Comprehensive comparison of all state-of-the-art MIP models.
- Systematic approach to symmetry breaking and valid inequalities.

**Weaknesses:**
- Only practical for very small instances (up to ~16 pieces reliably; cannot solve most instances with 25+ pieces to optimality).
- Computational complexity grows rapidly with number of pieces and geometric complexity (O(r^4 * n^2) binary variables).
- No support for rotations in the evaluated models.
- Strip packing formulation (not bin packing).
- Runtime can be hours even for small instances.

**Relevance to Thetta Project:** Exact MIP models are impractical for production nesting with realistic part counts (typically 50-500+ pieces). However, they serve two important roles: (1) providing provably optimal solutions for small instances to validate heuristic quality, and (2) the geometric decomposition concepts (vertical slices, convex NFP parts) can inform the design of heuristic placement strategies. The open-source RAMNEST library could serve as a reference implementation.

---

## Paper 6: GFPack++: Improving 2D Irregular Packing by Learning Gradient Field with Attention

**Title and Authors:** "GFPack++: Improving 2D Irregular Packing by Learning Gradient Field with Attention" by Tianyang Xue, Lin Lu, Yang Liu, Mingdong Wu, Hao Dong, Yanbin Zhang, Renmin Han, and Baoquan Chen.

**Year and Publication Venue:** 2024, arXiv preprint (arXiv:2406.07579v1).

**Problem Addressed:** 2D irregular packing with continuous rotation support and arbitrary container boundaries. The goal is to learn a placement policy from teacher packing examples using a score-based diffusion model.

**Method/Algorithm:**
1. **Problem Representation:** Each packing instance is represented as a set of objects with positions, orientations, and geometric features, plus container boundary conditions.
2. **Attention-Based Geometry Encoding:** Each polygon's geometry is encoded using an attention mechanism that captures both local vertex features and global shape characteristics. The boundary of the container is similarly encoded.
3. **Attention-Based Relation Encoding:** A transformer-based architecture learns the spatial relationships between all objects in a packing layout, capturing inter-object and object-container interactions.
4. **Gradient Field Learning:** The model learns a score function (gradient of the log-probability density) of good packing configurations. This is a diffusion/score-matching approach where:
   - Teacher layouts are used as training data.
   - The model learns to predict the gradient that pushes random configurations toward good packing layouts.
   - At inference, starting from random initial placements, the gradient field guides objects toward collision-free, compact configurations via Langevin dynamics.
5. **Collision Resolution:** Post-processing steps resolve remaining collisions after gradient descent.
6. **Continuous Rotation:** Rotations are represented as continuous variables and naturally handled by the diffusion process.

**Key Innovations:**
- First application of attention-based diffusion models to 2D irregular packing.
- Supports continuous rotation (not just discrete 0/90/180/270).
- Handles arbitrary (non-rectangular) container boundaries.
- Learns complex geometric relationships between pieces and containers from examples.
- Significantly faster than previous learning-based methods.

**Results:** Outperforms previous learning-based methods (GFPack, RL-based approaches) on both utilization and inference speed. Approaches but does not match the best traditional heuristic methods on standard benchmarks. Demonstrated on polygon packing, tangram puzzles, and texture atlas generation.

**Strengths:**
- Novel ML approach that learns from examples.
- Continuous rotation support.
- Arbitrary container boundaries.
- Parallelizable inference.
- Good generalization to unseen instances.

**Weaknesses:**
- Requires large training datasets of high-quality packing solutions.
- Does not consistently match traditional heuristic methods (like Guided Cuckoo Search) on established benchmarks.
- Post-processing collision resolution can be unreliable.
- Training is computationally expensive.
- Difficult to provide guarantees on solution quality.
- Black-box nature makes debugging difficult.

**Relevance to Thetta Project:** This represents the cutting edge of ML-based packing approaches. For the Thetta project, it is too early to adopt as the primary algorithm because: (1) traditional heuristics still outperform it; (2) it requires training data (chicken-and-egg problem); (3) reliability/determinism concerns for manufacturing. However, it could serve as a future enhancement layer -- for example, using a learned model for rapid initial placement that is then refined by traditional methods. The continuous rotation support is particularly relevant if Thetta needs to support arbitrary part orientations.

---

## Paper 7: MergeDJD: A Fast Constructive Algorithm with Piece Merging for the 2D Irregular Bin Packing Problem

**Title and Authors:** "MergeDJD: A Fast Constructive Algorithm with Piece Merging for the Two-Dimensional Irregular Bin Packing Problem" by Yi Zhou, Haocheng Fu, Yiping Liu, Jian Mao, Zhang-Hua Fu, and Yuyi Wang.

**Year and Publication Venue:** 2025 (arXiv preprint, dated February 28, 2026), submitted to Computers & Operations Research.

**Problem Addressed:** The 2D Irregular Bin Packing Problem (2DIBPP): pack irregular polygonal pieces into fixed-size rectangular bins, minimizing the number of bins used (or maximizing bin utilization).

**Method/Algorithm:**
1. **Piece Merge Preprocessing:**
   - Iteratively examine pairs of pieces to find those that can be combined into a single larger, more regular shape.
   - For each pair, try all rotations and vertex-to-vertex alignments to test if they can be merged without overlap and within bin bounds.
   - A fitness function evaluates merges based on three criteria:
     - `g_vtx`: Vertex count reduction (simpler shapes preferred).
     - `g_peri`: Perimeter reduction from overlapping edges (less boundary preferred).
     - `g_rect`: Rectangularity of combined shape (more rectangular preferred).
   - Merges exceeding a fitness threshold T are accepted. The process repeats until no more merges are possible.

2. **DJD Bin Packing Algorithm:**
   - Pieces (after merging) are sorted by decreasing area.
   - Each bin is filled greedily: place the largest fitting piece, then attempt groups of 1-3 pieces to fill remaining space.
   - A waste threshold gradually increases if no groups fit.

3. **ICAD Placement Heuristic (Improved CAD):**
   - Extends the CAD (Constructive Approach--Maximum Adjacency) placement strategy.
   - Augmented initial placement positions: adds all vertices of already-placed pieces as candidate positions, enabling placement inside concave regions.
   - Bidirectional sliding: uses both bottom-left and bottom-right slide strategies, selecting whichever produces higher adjacency (measured by the perimeter overlap metric).

**Key Innovations:**
- Piece merging as preprocessing: identifies geometrically compatible pieces and combines them before packing.
- ICAD: improves placement in concave regions through augmented candidate positions and bottom-right sliding.
- Specifically targets "cutting instances" where pieces were originally cut from a bin and have complementary edges.
- Open-source implementation on GitHub.

**Results:**
- Tested on 1,089 benchmark instances (JP1, JP2 jigsaw puzzles + standard nesting instances).
- Outperforms DJD on 1,083 of 1,089 instances.
- Achieves new best-known values on 515 instances.
- Very fast: sub-second runtimes for most instances, including the merge preprocessing.
- Particularly strong on cutting instances where pieces have complementary geometry.

**Strengths:**
- Extremely fast (constructive heuristic, not iterative search).
- Deterministic and reproducible.
- Open-source implementation available.
- The merge concept is original and practically useful.
- Strong performance on cutting instances common in manufacturing.

**Weaknesses:**
- As a constructive heuristic, solution quality does not match iterative improvement methods.
- The merge strategy is most effective when pieces have complementary edges (cutting instances); less impactful for arbitrary geometry.
- The fitness threshold T must be tuned.
- Does not support continuous rotations (only discrete angle set).

**Relevance to Thetta Project:** This is highly relevant to Thetta. The 2DIBPP (irregular pieces, fixed rectangular bins) is exactly the Thetta problem formulation. Key takeaways: (1) The piece-merge preprocessing is directly applicable when parts are cut from the same sheet or have complementary profiles. (2) The ICAD placement heuristic (augmented candidate positions + bidirectional sliding) is a practical placement strategy. (3) The open-source code (Java) can serve as a starting point or benchmark. (4) DJD's bin-by-bin greedy filling is a good baseline constructive heuristic for the Thetta pipeline. The F and K metrics defined in this paper are useful for evaluating Thetta's output quality.

---

## Paper 8: Sparrow: An Open-Source Heuristic to Reboot 2D Nesting Research

**Title and Authors:** "An open-source heuristic to reboot 2D nesting research" by Jeroen Gardeyn, Greet Vanden Berghe, and Tony Wauters.

**Year and Publication Venue:** 2025 (arXiv preprint, arXiv:2509.13329v3, dated February 2026).

**Problem Addressed:** 2D irregular strip packing: arrange irregular polygons on a fixed-width strip to minimize length. The paper also introduces 10 new real-world benchmark instances.

**Method/Algorithm:**
1. **Feasibility Problem Decomposition:** The optimization problem (minimize strip length) is decomposed into a sequence of feasibility problems: "Can all pieces fit within a strip of length L?"
2. **Binary Search on Strip Length:** The optimal length is found by binary search over candidate lengths, solving a feasibility problem at each step.
3. **Collision Resolution (Core Algorithm):**
   - Start with all pieces placed (possibly overlapping) on the strip.
   - Iteratively resolve collisions by moving pieces apart.
   - The resolution uses NFP-based overlap detection and compaction moves.
   - A key mechanism is "jostling" -- alternating between pushing pieces left-to-right and right-to-left to achieve tight packing.
4. **NFP-Based Geometry:** Uses no-fit polygons for overlap detection and feasible placement computation.
5. **Perturbation Mechanisms:** Various perturbation strategies (random piece removal and reinsertion, piece swaps, rotation changes) to escape local optima.
6. **Open Source:** The complete implementation is released as open-source software.

**Key Innovations:**
- Demonstrates that the field has stagnated not due to algorithmic limits but due to high barrier to entry and lack of reproducibility.
- Consistently outperforms the state-of-the-art (including Guided Cuckoo Search from 2013) -- in some cases by a wide margin.
- Open-source release directly addresses reproducibility crisis in the field.
- 10 new real-world benchmark instances for the community.
- Feasibility-based decomposition simplifies the optimization.

**Results:**
- Outperforms Guided Cuckoo Search (the previous state of the art from 2013) on virtually all standard benchmarks.
- Achieves significantly better utilization on many instances, sometimes by several percentage points.
- Demonstrates that a single well-implemented heuristic can surpass a decade of research.
- New best-known solutions on many benchmark instances.

**Strengths:**
- New state-of-the-art results on standard benchmarks.
- Open-source: enables reproducibility and serves as a foundation for future research.
- Relatively simple algorithm that is easy to understand.
- Practical performance with reasonable computation times.
- New benchmark instances from real industry.

**Weaknesses:**
- Focuses on strip packing (unbounded length), not bin packing (fixed rectangular bins).
- Implementation details may need adaptation for different problem variants.
- Binary search on strip length adds computational overhead.
- Performance depends on quality of NFP computation (uses existing NFP library).

**Relevance to Thetta Project:** This is one of the most important papers for the Thetta project. Although it addresses strip packing rather than bin packing, the core collision-resolution and compaction algorithm can be adapted for fixed bins. Key takeaways: (1) The feasibility decomposition approach (binary search on length, then solve feasibility) can be adapted for bin packing (fix bin size, maximize number of pieces placed). (2) The NFP-based collision resolution with jostling is a proven effective compaction strategy. (3) Being open-source, it can serve as a reference implementation or even be directly integrated into the Thetta pipeline. (4) The paper's new benchmark instances provide relevant test cases. Adapting Sparrow for fixed rectangular bins is a practical path to a high-quality nesting engine.

---

## Paper 9: A Heuristic for Solving the Irregular Strip Packing Problem with Quantum Optimization

**Title and Authors:** "A heuristic for solving the irregular strip packing problem with quantum optimization" by Paul-Amaury Matt and Marco Roth.

**Year and Publication Venue:** 2024, arXiv preprint (arXiv:2402.17542v1).

**Problem Addressed:** Irregular strip packing using quantum computing. The goal is to arrange irregular polygonal pieces within a fixed-height rectangular container to minimize waste.

**Method/Algorithm:**
1. **Problem Decomposition:** The strip packing problem is decomposed into two sub-problems:
   - **Ordering sub-problem:** Determine the sequence in which pieces should be placed (modeled as a Traveling Salesman Problem - TSP).
   - **Spatial arrangement sub-problem:** Determine the physical placement, modeled as a rectangle packing problem.
2. **Bounding Box Approximation:** Each irregular polygon is approximated by its axis-aligned bounding rectangle. This simplifies the packing to a rectangle packing problem.
3. **TSP for Ordering:** The piece ordering is solved using TSP formulation where the "distance" between pieces encodes how well they fit together in sequence.
4. **Quantum Optimization:**
   - **QAOA (Quantum Approximate Optimization Algorithm):** Applied to solve the TSP ordering problem.
   - **Quantum Alternating Operator Ansatz:** An alternative quantum approach also evaluated.
   - Both simulated quantum computers and real quantum hardware (IBM Quantum) are tested.
5. **Classical Placement:** After ordering is determined, pieces are placed sequentially using a bottom-left heuristic.

**Key Innovations:**
- First application of quantum computing algorithms to irregular strip packing.
- Novel decomposition into TSP (ordering) + rectangle packing (placement).
- Evaluation on both quantum simulators and real quantum hardware.
- Comparison between QAOA and Quantum Alternating Operator Ansatz.

**Results:**
- On small puzzle instances (6-12 pieces): quantum approaches find solutions, but quality is comparable to or worse than classical brute-force search.
- On standard benchmarks (SHAPES1, SHAPES2, SHIRTS): classical methods significantly outperform quantum approaches.
- Real quantum hardware results are worse than simulator results due to noise.
- The bounding box approximation wastes significant material compared to NFP-based approaches.

**Strengths:**
- Pioneering application of quantum computing to nesting.
- Clean problem decomposition.
- Thorough evaluation including real quantum hardware.
- Honest assessment of current quantum limitations.

**Weaknesses:**
- Results are significantly worse than classical state-of-the-art methods.
- Bounding box approximation is extremely lossy -- ignores the shape of pieces entirely.
- Current quantum hardware is far too noisy and small for practical nesting.
- Only works with very small instances (< 15 pieces).
- TSP formulation for ordering is an oversimplification.

**Relevance to Thetta Project:** This paper has minimal practical relevance to Thetta in the near term. Quantum computing for combinatorial optimization is still in its infancy, and the results here are far from competitive with classical approaches. The bounding box approximation discards the core challenge (irregular geometry). However, it is worth monitoring this space for the long term (5-10 years). The TSP-based ordering heuristic (as a classical algorithm, not quantum) could serve as a simple baseline for piece sequencing.

---

## Paper 10: Algorithms for Sheet Metal Nesting

**Title and Authors:** "Algorithms for Sheet Metal Nesting" by Jeffrey W. Herrmann and David R. Delalio.

**Year and Publication Venue:** University of Maryland, IEEE Transactions on Robotics and Automation (manuscript/technical report, approximately 2001).

**Problem Addressed:** The dynamic nesting decision problem for sheet metal punching: given a set of orders requiring the same material type and thickness, decide which orders to combine (nest) together on larger unsheared sheets to minimize total setup and material costs.

**Method/Algorithm:**
1. **Problem Formulation:** The nesting decision is formulated as an integer program. For each order, a binary variable indicates whether it is nested or processed individually. The objective minimizes the total cost (setup costs + material costs for both nested and unnested orders).
2. **NP-Completeness Proof:** The paper proves the problem is NP-complete via reduction from PARTITION.
3. **Dynamic Programming Solution:** A pseudo-polynomial time DP algorithm solves the problem exactly with complexity O(nA), where n is the number of orders and A is the usable sheet area.
4. **Linear Programming Relaxation:** A continuous LP relaxation is solved to obtain a lower bound.
5. **Simple Heuristic:** Derived from the LP relaxation optimal solution:
   - Calculate the approximate net cost T_j of nesting each order.
   - Let W = {orders with negative net cost}.
   - If the total net cost of nesting orders in W (including setup cost S) is negative, nest those orders. Otherwise, nest nothing.
6. **Worst-Case Bound:** The heuristic's cost exceeds optimal by less than R (the cost of one unsheared sheet).

**Key Innovations:**
- Formalization of the nesting decision as an optimization problem (not the layout problem, but which orders to batch together).
- NP-completeness proof for the nesting decision problem.
- Simple O(n) heuristic with provable worst-case error bound of less than one sheet cost.
- Analysis of nesting frequency impact on costs.

**Results:**
- The Simple heuristic achieves near-optimal solutions across all tested scenarios.
- Nesting everything or nothing can be far from optimal; the heuristic selects the right subset.
- Nesting frequency analysis shows that less frequent nesting (larger batches) generally reduces costs.

**Strengths:**
- Rigorous theoretical analysis with complexity proofs and worst-case bounds.
- Extremely simple and practical heuristic (can be implemented in a spreadsheet).
- Industry-grounded: developed with a real manufacturer.
- Clean separation of the "which orders to nest" decision from the "how to layout" decision.

**Weaknesses:**
- Assumes parts are small relative to sheets (area-based approximation for fitting).
- Does not address the geometric layout problem at all (assumes pieces fit if total area is sufficient).
- Only considers setup and material costs, not other manufacturing considerations.
- Limited to single-machine, same-material nesting decisions.
- Dated -- does not incorporate modern computational methods.

**Relevance to Thetta Project:** This paper addresses a higher-level decision problem: which orders/jobs to combine into a nesting batch. While the Thetta project primarily focuses on the geometric layout problem (given pieces, arrange them on bins), the batch decision problem is relevant for production planning. Key takeaway: the nesting decision (which orders to combine) should be separated from the layout decision (how to place pieces). The Simple heuristic provides a practical answer to "should we nest these orders together?" In a STEP-to-nest-to-STEP pipeline, this could inform a preprocessing step that decides which parts from different jobs should be combined onto the same sheet, balancing setup costs vs. material waste.

---

## Summary: Recommendations for the Thetta Project

### Architecture Recommendations
Based on this literature review, the Thetta nesting pipeline should follow this architecture:

1. **Geometry Extraction:** Extract 2D profiles from STEP face geometry.
2. **NFP Precomputation:** Compute pairwise No-Fit Polygons using Rocha (2019) or Burke (2007) algorithms. Also compute Inner-Fit Polygons (piece vs. rectangular bin).
3. **Placement Heuristic:** Use an ICAD-style bottom-left-fill strategy (from MergeDJD) with augmented candidate positions.
4. **Search/Optimization:** Adapt Sparrow's collision-resolution approach for fixed rectangular bins, with perturbation-based escape from local optima.
5. **Piece Merging (optional):** Apply MergeDJD's merge preprocessing when pieces have complementary edges (cutting instances).

### Most Impactful Papers for Implementation
1. **Sparrow (Paper 8):** Best-performing open-source nesting algorithm. Adapt for bin packing.
2. **MergeDJD (Paper 7):** Directly addresses 2DIBPP (irregular bin packing). Open-source, fast constructive heuristic.
3. **Burke (Paper 1) or Rocha (Paper 2):** For NFP computation (essential geometric primitive).
4. **Frontiers Review (Paper 3):** For understanding the field and identifying benchmarks.

### Papers of Limited Near-Term Relevance
- **Quantum (Paper 9):** Not competitive with classical methods.
- **GFPack (Paper 6):** ML-based, not yet matching traditional methods.
- **MIP/Vertical Slices (Paper 5):** Only practical for tiny instances; useful for validation.
- **UMD Sheet Metal (Paper 10):** Addresses batch decision, not layout.
- **Qin GA-SA (Paper 4):** Superseded by Sparrow's approach.
