"use client";

import Masonry from 'react-masonry-css';

// Mock image data with varied aspect ratios to mimic the inspiration image
const mockImages = [
  {
    id: "1",
    src: "https://picsum.photos/400/400?random=1",
    alt: "Square image 1"
  },
  {
    id: "2",
    src: "https://picsum.photos/400/600?random=2",
    alt: "Portrait image 1"
  },
  {
    id: "3",
    src: "https://picsum.photos/600/400?random=3",
    alt: "Landscape image 1"
  },
  {
    id: "4",
    src: "https://picsum.photos/400/500?random=4",
    alt: "Portrait image 2"
  },
  {
    id: "5",
    src: "https://picsum.photos/300/300?random=5",
    alt: "Small square"
  },
  {
    id: "6",
    src: "https://picsum.photos/400/800?random=6",
    alt: "Tall portrait"
  },
  {
    id: "7",
    src: "https://picsum.photos/500/400?random=7",
    alt: "Landscape image 2"
  },
  {
    id: "8",
    src: "https://picsum.photos/400/400?random=8",
    alt: "Square image 2"
  },
  {
    id: "9",
    src: "https://picsum.photos/350/450?random=9",
    alt: "Portrait image 3"
  },
  {
    id: "10",
    src: "https://picsum.photos/600/350?random=10",
    alt: "Wide landscape"
  },
  {
    id: "11",
    src: "https://picsum.photos/300/400?random=11",
    alt: "Portrait image 4"
  },
  {
    id: "12",
    src: "https://picsum.photos/450/300?random=12",
    alt: "Landscape image 3"
  }
];

export default function ImageGrid() {
  // Responsive breakpoints for masonry columns
  const breakpointColumnsObj = {
    default: 5,
    1280: 4, // xl
    1024: 3, // lg  
    768: 3,  // md
    640: 2,  // sm
    480: 2   // xs
  };

  return (
    <div className="p-0.5 md:p-1 lg:p-1">
      {/* Masonry layout with responsive columns */}
      <Masonry
        breakpointCols={breakpointColumnsObj}
        className="flex w-auto -ml-0.5 md:-ml-1 lg:-ml-1"
        columnClassName="pl-0.5 md:pl-1 lg:pl-0.5 bg-clip-padding"
      >
        {mockImages.map((image) => (
          <div
            key={image.id}
            className="relative group cursor-pointer rounded overflow-hidden bg-muted hover:shadow-lg transition-all duration-200 hover:scale-[1.02] mb-2 md:mb-3 lg:mb-2"
          >
            <img
              src={image.src}
              alt={image.alt}
              className="w-full h-auto object-cover"
              loading="lazy"
            />
            {/* Hover overlay */}
            <div className="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-200" />
          </div>
        ))}
      </Masonry>
    </div>
  );
}
