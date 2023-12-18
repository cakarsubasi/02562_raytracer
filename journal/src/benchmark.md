### Appendix: Raw Benchmark Results

This file includes the raw output of the benchmark file in case I left out some information within the report.

```sh
Benchmarking with 100 samples.

Performance scaling with triangles (1/4):
BVH: Teapot (6,320), 4, MT
  morton_codes:  174.9µs
  radix_sort:    99.994µs
  treelet_init:  68.982µs
  treelet_build: 236.276µs
  upper_tree:    263.86µs
  flattening:    149.058µs
  total:         993.07µs
BVH: Bunny (69,451), 4, MT
  morton_codes:  789.972µs
  radix_sort:    885.317µs
  treelet_init:  308.514µs
  treelet_build: 686.7µs
  upper_tree:    313.393µs
  flattening:    1.321042ms
  total:         4.304938ms
BVH: Dragon (871,414), 4, MT
  morton_codes:  9.81747ms
  radix_sort:    3.170978ms
  treelet_init:  4.180133ms
  treelet_build: 7.880445ms
  upper_tree:    419.181µs
  flattening:    23.81249ms
  total:         49.280697ms
----------------------------------

Performance scaling with maximum leaf primitives (2/4):
BVH: Dragon, 1, MT
  morton_codes:  10.747579ms
  radix_sort:    3.310354ms
  treelet_init:  4.281506ms
  treelet_build: 14.285238ms
  upper_tree:    429.017µs
  flattening:    68.385026ms
  total:         101.43872ms
BVH: Dragon, 2, MT
  morton_codes:  9.675215ms
  radix_sort:    2.988587ms
  treelet_init:  4.199119ms
  treelet_build: 13.021131ms
  upper_tree:    422.889µs
  flattening:    63.280614ms
  total:         93.587555ms
Dragon, 4, MT
  morton_codes:  9.81747ms
  radix_sort:    3.170978ms
  treelet_init:  4.180133ms
  treelet_build: 7.880445ms
  upper_tree:    419.181µs
  flattening:    23.81249ms
  total:         49.280697ms
BVH: Dragon, 6, MT
  morton_codes:  9.180374ms
  radix_sort:    2.889548ms
  treelet_init:  4.149981ms
  treelet_build: 5.422311ms
  upper_tree:    388.576µs
  flattening:    16.193306ms
  total:         38.224096ms
BVH: Dragon, 8, MT
  morton_codes:  9.237541ms
  radix_sort:    2.94198ms
  treelet_init:  4.110331ms
  treelet_build: 4.711496ms
  upper_tree:    381.411µs
  flattening:    12.35723ms
  total:         33.739989ms
BVH: Dragon, 16, MT
  morton_codes:  9.42948ms
  radix_sort:    3.05562ms
  treelet_init:  4.172411ms
  treelet_build: 3.5971ms
  upper_tree:    381.127µs
  flattening:    6.588801ms
  total:         27.224539ms
----------------------------------

Multithreaded performance scaling (3/4):
Dragon, 4, MT
  morton_codes:  9.81747ms
  radix_sort:    3.170978ms
  treelet_init:  4.180133ms
  treelet_build: 7.880445ms
  upper_tree:    419.181µs
  flattening:    23.81249ms
  total:         49.280697ms
BVH: Dragon, 4, ST
  morton_codes:  14.342445ms
  radix_sort:    8.757931ms
  treelet_init:  4.184123ms
  treelet_build: 48.622467ms
  upper_tree:    347.292µs
  flattening:    22.982654ms
  total:         99.236912ms
Dragon, 8, MT
  morton_codes:  9.237541ms
  radix_sort:    2.94198ms
  treelet_init:  4.110331ms
  treelet_build: 4.711496ms
  upper_tree:    381.411µs
  flattening:    12.35723ms
  total:         33.739989ms
BVH: Dragon, 8, ST
  morton_codes:  13.877526ms
  radix_sort:    9.273799ms
  treelet_init:  3.983207ms
  treelet_build: 29.752219ms
  upper_tree:    344.455µs
  flattening:    10.965513ms
  total:         68.196719ms
----------------------------------

Performance comparison with the BSP (4/4):

Teapot:
BVH: Teapot, 4, MT
  morton_codes:  174.9µs
  radix_sort:    99.994µs
  treelet_init:  68.982µs
  treelet_build: 236.276µs
  upper_tree:    263.86µs
  flattening:    149.058µs
  total:         993.07µs
BSP: Teapot, 4, dep: 20
  subdivision:   23.032083ms
  flattening:    8.43311ms
  total:         31.465193ms

Bunny:
BVH: Bunny , 4, MT
  morton_codes:  789.972µs
  radix_sort:    885.317µs
  treelet_init:  308.514µs
  treelet_build: 686.7µs
  upper_tree:    313.393µs
  flattening:    1.321042ms
  total:         4.304938ms
BSP: Bunny, 4, dep: 20
  subdivision:   128.489613ms
  flattening:    15.890057ms
  total:         144.37967ms

Dragon, 4 leaf primitives:
Dragon, 4, ST
  morton_codes:  14.342445ms
  radix_sort:    8.757931ms
  treelet_init:  4.184123ms
  treelet_build: 48.622467ms
  upper_tree:    347.292µs
  flattening:    22.982654ms
  total:         99.236912ms
Dragon, 4, MT
  morton_codes:  9.81747ms
  radix_sort:    3.170978ms
  treelet_init:  4.180133ms
  treelet_build: 7.880445ms
  upper_tree:    419.181µs
  flattening:    23.81249ms
  total:         49.280697ms
BSP: Dragon, 4, dep: 20
  subdivision:   795.177236ms
  flattening:    32.752233ms
  total:         827.929469ms

Dragon, 8 leaf primitives:
Dragon, 8, ST
  morton_codes:  13.877526ms
  radix_sort:    9.273799ms
  treelet_init:  3.983207ms
  treelet_build: 29.752219ms
  upper_tree:    344.455µs
  flattening:    10.965513ms
  total:         68.196719ms
Dragon, 8, MT
  morton_codes:  9.237541ms
  radix_sort:    2.94198ms
  treelet_init:  4.110331ms
  treelet_build: 4.711496ms
  upper_tree:    381.411µs
  flattening:    12.35723ms
  total:         33.739989ms
BSP: Dragon, 8, dep: 20
  subdivision:   780.522113ms
  flattening:    30.659233ms
  total:         811.181346ms
----------------------------------

All done.
```