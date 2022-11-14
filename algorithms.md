# Algorithms

This doc details the different algorithms as flowcharts.

## Global Algorithm

Here's the global algorithm:

```mermaid

    graph TD;

        start([Start]);
        input[Load Input File];

        img[/Image/];
        merge[Merge Images];
        grayscale[Get Grayscale Image];
        shadeRegions[Find Shade Regions];
        shadeGradients[Find Shade Gradients];
        straightLines[Make Straight Lines Image];

        maploc[/Map Location/];
        mapImg[Map Image - Google Maps API];

        out[/Output Image/];
        ends([End]);

        start --> input;

        input --> maploc;
        maploc --> mapImg;
        mapImg --> merge;

        input --> img;
        img --> merge;
        merge --> grayscale;
        grayscale --> shadeRegions;
        grayscale --> shadeGradients;
        shadeRegions --> straightLines;
        shadeGradients --> straightLines;


        straightLines --> out;
        out --> ends;


```

## Map Image

TO DO

## Merge Images

TO DO

## Grayscale Image Algorithm

Uses image crate's into_luma_alpha8 function.

## Shade Regions Determination Algorithm

Here's the algorithm to find the shade regions as a flowchart:


```mermaid

    graph TD;

    init([Init. Region Structs Vec]);

    pxlShadeApproxLoop[Pixel Shade Approximation Loop];
    pxlShadeDet[Pixel Shade Determination];
    pxlShadeUpdate[Pixel Shade Update in Struct];

    pxlAllocInit[Init. Pixel Allocated Vec];

    nextPixel[Next Pixel];
    pxlAllocLoop[Pixel Allocation Loop];
    pxlAllocDec{Pixel <br>Allocated?};
    newRegion[New Region];
    pxlAlloc[Allocate Pixel to New Region];
    pxlAllocNeighborCheck{All Neighbors <br> Allocated?}
    pxlAllocNeighbors[Neighbor Pixels Allocation <br> Recursive];
    pxlAllocNeighborsShadeCheck{Neighbor <br> Pixel Shade <br> == <br> Pixel Shade?};
    pxlAllocNeighbor[Allocate Neighbor Pixel to New Region];

    return([Returns Region Structs Vec]);

    init --> pxlShadeApproxLoop
    pxlShadeApproxLoop -.-> pxlShadeDet
    pxlShadeDet -.-> pxlShadeUpdate
    pxlShadeUpdate -.-> pxlShadeApproxLoop
    pxlShadeApproxLoop --> pxlAllocInit
    
    pxlAllocInit --> pxlAllocLoop
    pxlAllocLoop -.-> pxlAllocDec
    pxlAllocDec -.-> |yes|nextPixel
    pxlAllocDec -.-> |no|newRegion
    newRegion -.-> pxlAlloc
    pxlAlloc -.-> pxlAllocNeighbors
    pxlAllocNeighbors -.-> pxlAllocNeighborCheck
    pxlAllocNeighborCheck -.-> |yes|nextPixel
    pxlAllocNeighborCheck -.-> |no|pxlAllocNeighborsShadeCheck
    pxlAllocNeighborsShadeCheck -.-> |yes|pxlAllocNeighbor
    pxlAllocNeighborsShadeCheck -.-> |no|pxlAllocNeighbors
    pxlAllocNeighbor -.-> pxlAllocNeighbors
    pxlAllocLoop --> return

```

## Shade Regions Average Minimum Gradient Directions Algorithm

Here's the algorithm to find the shade gradients as a flowchart (*: Inner flowcharts bellow):


```mermaid

    graph TD;

    init([Find Size of Subpixels Matrix])

    dirStep[Calculate Directions Angular Step]
    dirEvalVecLoop[Calculate Directions/Angles to Evaluate]
    dirIdx[Get Direction Index]
    dir[Calculate Direction]

    initMinGradDirMap[Initialize Min. Grad. Directions Map]
    minGradDirMapLoop[Fill Min. Grad. Directions Map]
    refPixelCoord[Get Reference Pixel Coordinates]

    minGradDir[Get Direction of Minimal Gradient]

    subpxl[Get Subpixels*]
    dirGradLoop[Calculate Gradient for Each Direction*] 

    regsAvgMinGradDir[Determine Regions Average <br> Min. Gradient Directions Map]

    getCurRegion[Get Current Region]
    getCurPxlGradDir[Get Pixels' Min. <br> Gradient Directions]
    calcAvgGradDir[Calculate Pixels' Average <br> Min. Gradient Direction]

    return([Shade Regions with Average <br> Min. Gradient Directions])

    init --> dirStep
    dirStep --> dirEvalVecLoop

    dirEvalVecLoop -.-> dirIdx
    dirIdx -.-> dir
    dir -.-> dirEvalVecLoop

    dirEvalVecLoop --> initMinGradDirMap
    initMinGradDirMap --> minGradDirMapLoop

    minGradDirMapLoop -.-> refPixelCoord
    refPixelCoord -.-> subpxl
    subpxl -.-> dirGradLoop
    dirGradLoop -.-> minGradDir
    minGradDir -.-> minGradDirMapLoop

    minGradDirMapLoop --> regsAvgMinGradDir

    regsAvgMinGradDir -.-> getCurRegion
    getCurRegion -.-> getCurPxlGradDir
    getCurPxlGradDir -.-> calcAvgGradDir
    calcAvgGradDir -.-> regsAvgMinGradDir

    regsAvgMinGradDir --> return

```

```mermaid

    graph TD;

    subpxl[Get Subpixels*]
    nLayers[Get Number of Layers]
    detPxlCoordRng[Determine Pixel <br> Coordinate Ranges]
    fillSubpxl[Add Pixels to Subpixels]

    subpxl -.-> nLayers
    nLayers -.-> detPxlCoordRng
    detPxlCoordRng -.-> fillSubpxl
    fillSubpxl -.-> subpxl

```

```mermaid

    graph TD;

    dirGradLoop[Calculate Gradient for Each Direction*]
    getPxlLine[Get Pixels in Direction]
    calcDiff[Calculate Differentials]
    calcGrad[Calculate Gradient]

    dirGradLoop -.-> getPxlLine
    getPxlLine -.-> calcDiff
    calcDiff -.-> calcGrad
    calcGrad -.-> dirGradLoop

```
## Straight Lines Image Algorithm


```mermaid

    graph TD;

    init([Initialize Output Image]);

    strokeWidth[Get Stroke Width]
    regionsLoop[Shade Regions Loop]
    getRegionShade[Get Region Average Shade]
    getRegionGradDir[Get Region Average <br> Minimum Gradient Direction]
    pxlLoop[Pixel Painting Loop]
    pxlCoord[Get Pixel Coordinates]
    calcBlackStrokeWidth[Calculate Black Stroke Width]
    calcIdxShadeStroke[Calculate Index of Pixel in Stroke]
    decisionShadeStroke{Index of Pixel in Stroke <br>< Black Stroke Width?}
    pxlBlack[Paint Pixel Black]
    pxlWhite[Paint Pixel White]

    return([Return <br> Output Image]);

    init
    --> strokeWidth
    --> regionsLoop
        -.-> getRegionShade
        -.-> getRegionGradDir
        -.-> pxlLoop
            -.-> pxlCoord
            -.-> calcBlackStrokeWidth
            -.-> calcIdxShadeStroke
            -.-> decisionShadeStroke
                decisionShadeStroke -.-> |yes|pxlBlack -.-> pxlLoop
                decisionShadeStroke -.-> |no|pxlWhite -.-> pxlLoop
        -.-> regionsLoop
    --> return 

```