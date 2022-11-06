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

## Shade Gradients Algorithm

TO DO (already implemented)

## Straight Lines Image Algorithm

TO DO