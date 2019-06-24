Clustered Light Shading for Stereoscopic Rendering

To render scenes with many light sources in real-time, we need to reduce the
number of shading operations per pixel. Clustered light shading divides the
frustrum into clusters and assigns lights to them. Each fragment looks up its
cluster and iterates only over the lights that are likely to have an impact. We
demonstrate that with some adjustments, the clusters can be re-used to render
the scene from both eyes. 
