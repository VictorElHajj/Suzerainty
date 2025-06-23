Tectonics notes:
Subduction: The thinner of the two plates goes under always (ocean plates are always thinner than land plates, maybe add a random variation within that too)
Causes border tiles on the thin side to become lower and on the thick side to become higher

Idea:

Start by assigning tiles to continents, with a initial pangea of connected continental plates surrounded by ocean plates.
Each place is actually detached from the grid, the points rotate along the sphere. Use some fluid dynamics -ish logic so that the points making up the continents are pulled together if drifting apart and away if too close, and when points of two different continents interact use the above logic.

Plate types: 
Continental: Thick, 10-70km, felsic, low density
Oceanic: Thin, 5-10km, mafic, high density

## Boundary types:

### Convergent: Plates coming together
Oceanic-Continental: Oceanic slides under continental, causing hight decrease in oceanic and height increase in continental. Causes mountains and volcanoes. Like the Andes.
Oceanic-Oceanic: Same but under water, might cause islands and volcanoes.
Continental-Continental: Creates very big mountains, neither plate will go under the other.

### Divergent: Plates drifting apart
Oceanic-Oceanic: New crust is created, created under water mountains and some islands (so height increases but not as much).
Continental-Continental: New crust is created, creates mountains but not as tall or much
Oceanic-Continental: As new crust is created this effectively becomes oceanic-oceanic

### Transform: Plates sliding past one another.
Continental-Continental: Earthquakes, no height difference
Oceanic-Oceanic: Earthquakes, no height difference
Abstract away into Divergent/Convergent but weaker depending on directions.

Use particles to model the continents and the interactions should hote plates together but also keep points from gettign too close. Particle interactions between plates should cause height to go up and down.

Thermal erosion and wind erosion will then also moderate this.

How to optimize particle lookups? Divide sphere into buckets again, and check all buckets within the particle radius. 


### Current order
1. plate rotation axis needs to change over time
2. Replace hard coded plate velocity with a plate force and breaking force
3. Write vertex update method interpolating from plate particles
