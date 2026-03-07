# GFPack++ Analysis

## 1. Overview

GFPack++ is a machine learning-based approach to 2D irregular packing that uses attention-driven gradient fields (score-based diffusion models) to learn optimal packing configurations. Published at ICCV 2025 (International Conference on Computer Vision), it is a research system from Shandong University, Microsoft Research Asia, and Peking University. The approach learns gradient fields from teacher demonstrations (pre-computed good packing solutions) and uses SDE-based sampling (reverse diffusion) to generate new packing layouts at inference time.

This is a successor to the original GFPack (SIGGRAPH Asia 2023), adding attention-based geometry encoding, continuous rotation support, and arbitrary boundary generalization.

## 2. Architecture

Python + PyTorch research codebase with native C++ extensions:

```
src/
  __init__.py
  score_model.py      - PolygonPackingTransformer: the main score model
  gnn_feature.py      - PolygonGCN: GNN-based polygon feature extractor
  sde.py              - SDE definitions (VE/VP/sub-VP), loss function, PC sampler
  geometry_utils.py   - Polygon class, sweep-line grid rasterizer, GNN data creation
  train.py            - Distributed training entry point (torchrun/DDP)
  evaluate.py         - Evaluation/inference pipeline
scripts/
  train.sh            - Training shell wrapper
  eval.sh             - Evaluation shell wrapper
  genPuzzle.py        - Dataset puzzle generation
  make_video.py       - Result visualization video
external/
  rmspacing_cpp/      - C++ pybind11 extensions
    main.cpp          - rmspacing: overlap resolution with boundary constraints
    util.cpp          - calutil: utility/overlap calculation
    clipper.*         - Bundled Clipper2 library for polygon operations
    AABB.hpp          - Axis-aligned bounding box
    build.sh          - Builds rmspacing.so
    build_util.sh     - Builds calutil.so
data/
  datasets/           - Pickled training/validation datasets
  polygons/           - Polygon definition files (vertex lists)
checkpoints/          - Pretrained model weights
logs/                 - TensorBoard training logs
```

**Key files:**
- `src/score_model.py` - Transformer-based score model (92 lines, core architecture)
- `src/gnn_feature.py` - Graph neural network polygon encoder (107 lines)
- `src/sde.py` - SDE framework, loss function, PC sampler (272 lines)
- `src/evaluate.py` - Evaluation with utility calculation and spacing removal
- `src/train.py` - Distributed training loop
- `external/rmspacing_cpp/util.cpp` - Native overlap/utility calculation

## 3. Algorithm Details

### Score-Based Diffusion Model
GFPack++ formulates packing as a score-based generative modeling problem:

1. **State representation**: Each polygon's placement is a 4D vector `[x, y, cos(theta), sin(theta)]` (position + rotation encoding)
2. **Forward SDE (noising)**: Gradually adds noise to good packing configurations (teacher solutions)
3. **Score model**: Learns the gradient (score) of the log-probability of good configurations
4. **Reverse SDE (denoising)**: Starting from random noise, iteratively denoise to produce valid packing layouts

### Score Model Architecture (`PolygonPackingTransformer`)
- **Time embedding**: Gaussian Fourier Projection -> MLP (encodes diffusion timestep)
- **Action encoder**: MLP (4D action -> feature_dim), encodes current polygon positions
- **Geometry feature**: PolygonGCN extracts per-polygon geometric features
- **Transformer**: Full encoder-decoder Transformer (default: 8 encoder + 8 decoder layers, 16 heads, hidden_dim=128)
  - Input: concatenation of [geometry_features, action_features, time_embedding]
  - Padding mask for variable-count polygons
- **Output**: Linear projection to 4D velocity (dx, dy, d_cos, d_sin), normalized by noise std

### Polygon GNN Feature Extractor (`PolygonGCN`)
- Input: polygon vertices as graph nodes (features: x, y, internal_angle)
- 4-layer GCN with DenseNet-style concatenation (skip connections)
- Global features: perimeter encoded via linear layer, concatenated per-node
- 2-layer Transformer encoder for sequence-aware processing
- Attention pooling to produce fixed-size per-polygon embedding

### SDE Framework
Three SDE variants supported:
- **VE (Variance Exploding)**: sigma_min=1.0, sigma_max=1000.0
- **VP (Variance Preserving)**: beta_0=0.1, beta_1=20
- **sub-VP**: Variance preserving with discount

### Training
- Denoising score matching loss: `||score * std + noise||^2`
- Weighted by sample importance (`state.w`)
- Distributed training via `torchrun` / DDP
- EMA (Exponential Moving Average) of model weights
- Checkpointing + TensorBoard logging

### Inference (PC Sampler)
`pc_sampler_state()`:
1. Initialize random positions for all polygons
2. 128 timesteps from t=1.0 to t=0.01
3. At each step: compute score -> apply SDE drift + diffusion noise
4. Step size scaled by 14x (hyperparameter)
5. Final positions decoded to translations + rotations

### Post-Processing
- **rmspacing**: C++ module that resolves overlaps while respecting boundary constraints (width/height)
- **calutil**: C++ module that computes utilization metrics (sum area, max intersection, bounding box)
- Utilization = sum_polygon_area / bounding_box_area

## 4. Dependencies

**Python (conda environment `gfpack`):**
- **PyTorch** 1.11.0 + CUDA 11.3
- **PyTorch Geometric (pyg)** 2.0.4 - GNN framework (GCNConv, Data, Batch)
- **pytorch-scatter/sparse/cluster** - PyG dependencies
- **Shapely** 2.0.7 - Polygon area/perimeter computation
- **NumPy** 1.26.4 + **SciPy** 1.13
- **matplotlib** 3.9 - Visualization
- **TensorBoardX** 2.6 - Training logging
- **OpenCV** 4.11 - Image processing
- **pybind11** 2.13 - C++ extension building
- **aabbtree** 2.8 - AABB tree for spatial queries
- **scikit-learn** 1.6

**Native C++ extensions:**
- **Clipper2** (bundled) - Polygon boolean operations for overlap detection
- Custom `rmspacing` module - Overlap resolution
- Custom `calutil` module - Utility calculation

Python 3.9, CUDA 11.3 required.

## 5. Strengths

- **Novel approach** - First attention-based diffusion model for 2D irregular packing (ICCV 2025)
- **Continuous rotation** - Supports arbitrary rotation angles via sin/cos encoding, not limited to discrete steps
- **Generalization** - Trained model generalizes to unseen polygon shapes and arbitrary boundaries
- **Fast inference** - Once trained, inference is orders of magnitude faster than conventional metaheuristic approaches
- **Rich geometry encoding** - GCN with DenseNet skip connections + Transformer encoder captures complex polygon features
- **Scalable** - Handles variable numbers of polygons via padding masks
- **Principled framework** - SDE/score matching provides a mathematically grounded generative framework
- **Pretrained checkpoints** - Available for immediate use

## 6. Weaknesses

- **Training data dependency** - Requires pre-computed "teacher" demonstrations (good packing solutions) as training data
- **GPU required** - CUDA 11.3 mandatory; no CPU-only path
- **Older PyTorch** - Locked to PyTorch 1.11 (2022); may have compatibility issues with modern GPUs
- **Post-processing needed** - Raw model output may have overlaps; requires `rmspacing` C++ module for valid solutions
- **No guaranteed feasibility** - Diffusion sampling doesn't guarantee overlap-free solutions natively
- **Complex setup** - Conda environment with specific CUDA/PyG versions, plus compiling C++ extensions
- **Research code quality** - Chinese comments, commented-out debug prints, hardcoded paths, duplicated functions between train.py and evaluate.py
- **Limited polygon count** - `maxInputLength=50` in the Transformer; may struggle with larger instances
- **Fixed evaluation workspace** - `rmspacing` uses hardcoded width=8000; boundary parameters must be aligned
- **No interactive tool** - Pure batch processing; no GUI or real-time feedback
- **Dataset format** - Requires specific pickle format for training data; polygon definitions in custom text format

## 7. Key Code Paths

**Training:**
`src/train.py` -> `collectData()` loads pickled dataset -> `lossFun()` in `sde.py`:
1. Sample random timesteps
2. Perturb true positions with noise scaled by marginal_prob std
3. Score model predicts velocity
4. Loss = weighted denoising score matching

**Inference:**
`src/evaluate.py` -> `pc_sampler_state()` in `sde.py`:
1. Initialize random 4D positions for each polygon
2. 128 reverse SDE steps: score model predicts gradient -> apply drift + diffusion
3. Output: batch of candidate packing configurations
4. Post-process: `removeSpacing()` calls `rmspacing.rm_spacing_all()` (C++)
5. Evaluate: `cal_util()` calls `calutil.cal_util_all()` (C++) for utilization metrics

**Score model forward pass:**
`PolygonPackingTransformer.forward()` in `src/score_model.py`:
1. Embed timestep via Gaussian Fourier Projection
2. Encode actions (positions) via MLP
3. Extract polygon geometry features via PolygonGCN
4. Concatenate [geo_features, action_features, time_embedding]
5. Pass through Transformer encoder-decoder with padding masks
6. Output layer projects to 4D velocities, normalized by noise std

**GNN feature extraction:**
`PolygonGCN.forward()` in `src/gnn_feature.py`:
1. Node features: [x, y, internal_angle] per vertex
2. 4 GCN layers with DenseNet concatenation
3. Global features (perimeter) concatenated
4. Transformer encoder for context
5. Attention pooling to fixed-size embedding
