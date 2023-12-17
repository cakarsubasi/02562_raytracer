# Worksheet 4: Radiometric and Photometric exercises

# Part 1

> A small 25 W light bulb has an efficiency of 20%. How many photons are approximately emitted per second? Assume in the calculations that we only use average photons of wavelength 500 nm.

The energy of a single photon can be calculated with:

\\[E_{photon} = \frac{hc}{\lambda}\\]

Where \\(h\\) is the Planck constant, \\(c\\) is the speed of light in a vacuum, and \\(\lambda\\) is the wavelength. \\(hc\\) has a constant value of around \\(2\times10^{-25}Jm\\)

So in this case:

\\[E_{photon}\approx 4\times10^{-19}J\\]

The number of photons can be found with:

\\[n=\frac{P\eta}{E_{photon}}\\] 

per unit time where \\(\eta\\) is the efficiency.

Therefore the number of photons is approximately \\(1.25\times10^{19}\frac{1}{s}\\).

# Part 2

>A light bulb (2.4 V and 0.7 A), which is approximately sphere-shaped with a diameter of 1 cm, emits light equally in all directions. Find the following entities (ideal conditions assumed)
>
> - Radiant flux
> - Radiant intensity
> - Radiant exitance
> - Emitted energy in 5 minutes
>
>Use \\(W\\) for Watt, \\(J\\) for Joule, \\(m\\) for meter, \\(s\\) for second and \\(sr\\) for steradian

- Energy: \\(Q = P \times t\\)

- Radiant Flux (Power): \\(\Phi = \frac{dE}{dt} = IV\\)

\\(2.4V \times 0.7A = 1.68 W = 1.68 J s^-1\\)


- Radiant Intensity: \\(I = \frac{d\Phi}{d\omega}\\)

We have the entire sphere which means \\(\Omega\\) is \\(4\pi sr\\)

\\(I = 0.1337 \frac{W}{sr}\\)

Radiant exitance just requires us to convert from solid angles to actual surface area.

Radiant Exitance: \\(M = \frac{d\Phi}{dA} = \frac{I}{r^2}\\)

 \\(M = \frac{0.1337}{0.1^2} = 1337 \frac{W}{m^2}\\)

Emitted energy in 5 minutes: \\(P\times 300s\\)

\\(1.68 \times 300 = 504 Wh \\)

# Part 3

> The light bulb from above is observed by an eye, which has an opening of the pupil of 6 mm and a distance
of 1 m from the light bulb. Find the irradiance received in the eye.

We need to find the solid angle from the sphere to the opening. It is 1 meter from the center of the light source to the pupil, if the pupil covered the entire halfsphere, it would be about \\(2\pi\\) in length. So, the 2D angle from the center should be:

\\( 0.006 / 2\pi \\)

We actually need half of this to return to \\( \theta \\).

Then we can take the lazy route and use the formula I found in Wikipedia to convert this to Stradians. Integrals are for people who are good at writing \\( \LaTeX \\) equations.

\\( \Omega = 2\pi(1 - cos(\theta)) \\)

We then divide radiant intensity with this value converted to area:

\\(E = \frac{\Phi}{2\pi(1 - cos(\theta))r^2} = \frac{1.68}{2\pi(1-cos(0.003/2\pi))0.01^2} = \\)

# Part 4

> A 200 W spherically shaped light bulb (20% efficiency) emits red light of wavelength 650 nm equally in all
directions. The light bulb is placed 2 m above a table. Calculate the irradiance at the table.
>
> Photometric quantities can be calculated from radiometric ones based on the equation
>
>\\[Photometric = Radiometric · 685 · V (λ)\\]
>
>in which V (λ) is the luminous efficiency curve. At 650 nm, the luminous efficiency curve has a value of 0.1. Calculate the illuminance.

Illuminance:

# Part 5

> In a simple arrangement the luminous intensity of an unknown light source is determined from a known light source. The light sources are placed 1 m from each other and illuminate a double sided screen placed between the light sources. The screen is moved until both sides are equally illuminated as observed by a photometer. At the position of match, the screen is 35 cm from the known source with luminous intensity Is = 40 lm/sr = 40 cd and 65 cm from the unknown light source. What is the luminous intensity Ix of the unknown source?

Luminous intensity:

# Part 6

> The radiance L from a diffuse light source (emitter) of 10×10 cm is \\(5000 \frac{W}{m^2 sr}\\). Calculate the radiosity (radiant exitance). How much energy is emitted from the light source?

Radiosity:

Energy:

# Part 7

> The radiance \\(L = 6000 cos(θ) \frac{W}{m^2 sr} \\) for a non-diffuse emitter of area 10 by 10 cm. Find the radiant exitance. Also, find the power of the entire light source.

Radiosity:

Power:


