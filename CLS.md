# Clustered Light Shading

## Scenario

Let us assume that the lights are at least culled to the view frustrum, so we
are trying to be faster than shading every pixel with all lights in the view
frustrum. Not all lights in the scene because that would be silly.

## Realizations

1. Assuming lights are distributed somewhat uniformly over the scene, it makes
   sense to cluster in pre-projective space.
   
2. The perspective view frustrum divided by the orthographic view frustrum
   approaches 1/3 as z1 - z0 grows large. This means I can reduce the number of
   light assigments and memory usage to 1/3rd.
   
3. If we divide up the clustering space in the z-axis (many cascades), we can approach a pyramid.

4. If we can determine the min and max of x and y for every layer in the pyramid
   we can reduce the assigments and memory requirements even more. I have a
   feeling this reduces the number of clusters by quite a lot, except for scenes
   with long diagonal geometry?
   
   However this means we have to iterate and do the clipping on all geometry on
   the CPU or readback from the GPU. We can do this asynchronously and add a
   large margin or maybe we can do the light assignment on the GPU.
   
   Having layers adds an indirection to the cluster lookup. I expect this is not
   much of a problem because the layer headers should be small and stay
   resident in some cache.

## Min/Max layers

We want to compute for all views, for all geometry, for every layer in the
pyramid, the min and max positions (bounds of the layer) in clustering space.
This is kind of rediculous.

We want to clip in eye space but emit the cls space positions...

To get the number of layers we would first h ave to determine z_min and z_max
for every view.

The we determine the number of layers Nz = floor((z_max - z_min)/cluster_side).

Can we emit the min/max of post-clipping triangles on the GPU? The z value
determines the layer and then the min and max x, y, and z are emitted and
blended with GL_BLEND_MIN/ max?


## Questions

1. How much data can I realistically send to the GPU per frame? What is my
   budget?
3. How can you do light assignment on the GPU?
3. Any ideas on how to compute the min/max per layer on the GPU efficiently?
4. How do I make u16 integers work on the GPU?


## Variations

### Clustering

1. Clustered orthographic
2. Clustered layered frustrum bounded orthographic
3. Clustered layered fragment bounded orthograhpic
4. Tiled

### Light assignment

CPU: for each light, for each cluster
CPU: for each cluster, for each light
GPU: for each light, for each cluster?
GPU: for each cluster, for each light (markus did this with bounding volume hierarchy)?

## Metrics
1. cluster count
2. cluster dimension computation time
3. light assignment count, time
4. shading operation count, time
5. acceleration data structure bytes

## Light density and radii vs cluster size

What is the optimal cluster size? What are we trying to achieve? 

We only want to iterate over the lightsk
