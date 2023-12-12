## Worksheet 10 - Blender

Ah yes, Blender my old friend. Unfortunately I am more in the modelling side of things (I wrote a whole Bachelor's thesis on it), so this part was more of an obligation unfortunately.

Create a cube, hit render:

![](./img/w10_e0a.png)

Load an environment map, hit render:

![](./img/w10_e0b.png)

Add a quad underneath the cube, set the rendering options to mask everything except shadows, change camera angle a bit, hit render:

![](./img/w10_e0c.png)

And we did not even make it to the first task!

### 1. Sphere and Cube

Add a sphere and change the materials a bit.

![](./img/w10_e1.png)

Due to the way the environmental mapping works, any reflection that is not on a sphere ends up looking very strange as the rays hit our objects as if they come from infinitely far away.

### 2. Principled BSDF

Change some more parameters and change the camera angle again.

![](./img/w10_e2.png)

I kind of feel I should look into how I can add some blur to the rendered objects to mask them being obvious fake additions to the scene which is clearly way grainier resolution than our pristine objects.

### 3. "Cool" Scene

Not done.

### 4. Discussion

It is worth noting that Cycles is an extremely powerful production ready offline renderer that can scale to multiple machines.

If it is not enough for someone, may I suggest something like MoonRay @@MOONRAY:5 ?