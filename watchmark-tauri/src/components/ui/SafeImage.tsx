import React, { useState, useEffect } from 'react';
import { motion } from 'framer-motion';
import { Eye } from 'lucide-react';
import { formatImagePath } from '../../utils/imageFormat';

interface SafeImageProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  srcPath: string;
  type: 'poster' | 'backdrop' | 'still';
  altText: string;
  className?: string;
  title?: string;
  fallbackSrcPath?: string;
  episodeNumber?: number;
  releaseDate?: string;
  isDateKnown?: boolean;
  isExactDate?: boolean;
  isFallbackImage?: boolean;
  potentialSpoiler?: boolean;
  isCompleted?: boolean;
}

export const SafeImage: React.FC<SafeImageProps> = ({ srcPath, type, altText, className, title, fallbackSrcPath, episodeNumber, releaseDate, isDateKnown, isExactDate, isFallbackImage, potentialSpoiler, isCompleted, ...rest }) => {
  const [imgSrc, setImgSrc] = useState<string | null>(null);
  const [hasError, setHasError] = useState(false);
  const [fallbackFailed, setFallbackFailed] = useState(false);
  const [fallbackSrc, setFallbackSrc] = useState<string | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const [isHovering, setIsHovering] = useState(false);
  const [isRevealed, setIsRevealed] = useState(false);

  useEffect(() => {
    let timer: any;
    if (isHovering && potentialSpoiler && !isCompleted && !isRevealed) {
      timer = setTimeout(() => {
        setIsRevealed(true);
      }, 1000);
    } else if (!isHovering) {
        setIsRevealed(false);
    }
    return () => clearTimeout(timer);
  }, [isHovering, potentialSpoiler, isCompleted, isRevealed]);

  useEffect(() => {
    if (fallbackSrcPath && fallbackSrcPath.trim() !== '') {
      try {
        if (fallbackSrcPath === 'gradient') {
          setFallbackSrc(null);
        } else {
          setFallbackSrc(formatImagePath(fallbackSrcPath, type === 'backdrop' ? 'w1280' : 'w500'));
        }
      } catch {
        setFallbackSrc(null);
      }
    } else {
      setFallbackSrc(null);
    }
  }, [fallbackSrcPath, type]);

  useEffect(() => {
    setHasError(false);
    setFallbackFailed(false);
    if (!srcPath || srcPath.trim() === '') {
      setHasError(true);
      return;
    }

    try {
      const formatted = formatImagePath(srcPath, type === 'backdrop' ? 'w1280' : 'w500');
      setImgSrc(formatted);
    } catch {
      setHasError(true);
    }
  }, [srcPath, type]);

  const handleImageError = (e: any) => {
    // If it was a local cached asset URL, attempt to recover by fetching directly from TMDB CDN before giving up
    if (imgSrc && (imgSrc.includes('asset.localhost') || imgSrc.includes('asset://') || imgSrc.includes('/cache/posters/') || imgSrc.includes('/cache/backdrops/') || imgSrc.includes('/cache/stills/'))) {
      const filenameMatch = imgSrc.match(/(?:w500|w342|w1280|original)_(?:pseudo_)?([^/?#]+)/i);
      if (filenameMatch && filenameMatch[1]) {
        const tmdbFilename = filenameMatch[1];
        const recoveryUrl = `https://image.tmdb.org/t/p/${type === 'backdrop' ? 'w1280' : 'w500'}/${tmdbFilename}`;
        if (imgSrc !== recoveryUrl) {
          setImgSrc(recoveryUrl);
          return;
        }
      }
    }

    try {
      e.target.style.opacity = '0';
      e.target.style.objectPosition = 'transparent';
      setHasError(true);
    } catch (err) {
      console.warn("Failed to set image error fallback state silently.");
    }
  };

  const handleFallbackError = (e: any) => {
    if (fallbackSrc && (fallbackSrc.includes('asset.localhost') || fallbackSrc.includes('asset://') || fallbackSrc.includes('/cache/'))) {
      const filenameMatch = fallbackSrc.match(/(?:w500|w342|w1280|original)_(?:pseudo_)?([^/?#]+)/i);
      if (filenameMatch && filenameMatch[1]) {
        const tmdbFilename = filenameMatch[1];
        const recoveryUrl = `https://image.tmdb.org/t/p/w1280/${tmdbFilename}`;
        if (fallbackSrc !== recoveryUrl) {
          setFallbackSrc(recoveryUrl);
          return;
        }
      }
    }

    try {
      e.target.style.opacity = '0';
      e.target.style.objectPosition = 'transparent';
      setFallbackFailed(true);
    } catch (err) {
      console.warn("Fallback image completely failed silently.");
    }
  };

  if (hasError || !imgSrc || (type === 'backdrop' && fallbackSrcPath === 'gradient')) {
    const year = isDateKnown
      ? (isExactDate ? (releaseDate ? releaseDate.substring(0, 4) : '') : releaseDate)
      : 'TBD';

    if (type === 'poster' || type === 'backdrop') {
      return (
        <div
          className={`relative overflow-hidden flex flex-col items-center justify-center text-center p-4 ${type === 'poster' ? 'rounded-xl' : ''} ${className || ''}`}
          style={{ background: 'linear-gradient(to bottom right, #1F222A, #0D0F14)' }}
        >
          <span className="text-white font-bold drop-shadow-md text-sm md:text-base md:text-2xl leading-tight break-words z-10 uppercase tracking-tight">
            {title || altText || 'Unknown Title'}
          </span>
          {year && year !== 'TBD' && (
            <span className="text-[#A0AEC0] font-bold drop-shadow-md text-xs md:text-lg mt-2 z-10 tabular-nums">
              {year}
            </span>
          )}
        </div>
      );
    } else if (type === 'still') {
      if (fallbackSrc && !fallbackFailed) {
        return (
          <div className={`relative overflow-hidden bg-[#0D0F14] ${className || ''}`}>
            <img
              src={fallbackSrc}
              alt="Backdrop Fallback"
              className="absolute inset-0 w-full h-full object-cover blur-[12px] brightness-50"
              onError={handleFallbackError}
            />
            <div className="absolute inset-0 bg-black/60"></div>
            <div className="absolute inset-0 flex items-center justify-center z-10">
               <span className="text-white font-bold text-3xl drop-shadow-lg tracking-wider uppercase opacity-90">
                 {episodeNumber ? `EP ${episodeNumber}` : (title || "EP")}
               </span>
            </div>
          </div>
        );
      }
      return (
        <div
          className={`relative overflow-hidden flex flex-col items-center justify-center text-center p-4 border border-[#2A2D35] ${className || ''}`}
          style={{ background: 'linear-gradient(to bottom right, #1F222A, #0D0F14)' }}
        >
          <span className="text-white font-bold text-3xl drop-shadow-lg tracking-wider uppercase opacity-90 z-10">
            {episodeNumber ? `EP ${episodeNumber}` : (title || "EP")}
          </span>
        </div>
      );
    }
  }

  if (type === 'backdrop') {
    return (
      <div className={`relative overflow-hidden z-0 ${className || ''}`} style={!isLoaded ? { background: 'linear-gradient(to bottom right, #1F222A, #0D0F14)' } : undefined}>
        <motion.img
          src={imgSrc!}
          alt={altText}
          className="absolute inset-0 w-full h-full object-cover z-10 origin-center"
          initial={{ opacity: 0 }}
          animate={{ opacity: isLoaded ? 1 : 0 }}
          transition={{ opacity: { duration: 0.5, ease: "easeInOut" } }}
          onLoad={() => setIsLoaded(true)}
          onError={handleImageError}
          {...rest as any}
        />
      </div>
    );
  }

  const shouldBlur = potentialSpoiler && !isCompleted && !isRevealed;
  const objectPositionStyle = type === 'still' ? { objectPosition: 'center 20%' } : {};

  const handleImageLoad = (e: React.SyntheticEvent<HTMLImageElement, Event>) => {
    setIsLoaded(true);
    if (type === 'still') {
      const img = e.currentTarget;
      if (img.naturalWidth > 0 && img.naturalWidth < 320) {
        img.classList.add('blur-[1px]', 'brightness-75');
      }
    }
  };

  return (
    <div
        className={`relative overflow-hidden ${className || 'w-full h-full'} ${type === 'still' ? 'aspect-video' : ''}`}
        onMouseEnter={() => setIsHovering(true)}
        onMouseLeave={() => setIsHovering(false)}
    >
      <motion.div
        className="w-full h-full absolute inset-0 z-0"
        initial={false}
        animate={{ filter: shouldBlur ? 'blur(20px)' : 'blur(0px)' }}
        transition={{ duration: 0.3 }}
      >
        <img
          src={imgSrc!}
          alt={altText}
          className={`w-full h-full object-cover ${isFallbackImage ? 'brightness-75' : ''}`}
          style={objectPositionStyle}
          onError={handleImageError}
          onLoad={handleImageLoad}
          {...rest as any}
        />
      </motion.div>

      {shouldBlur && (
         <div
           className="absolute inset-0 flex items-center justify-center z-20 cursor-pointer transition-colors"
           onClick={(e) => {
             e.preventDefault();
             e.stopPropagation();
             setIsRevealed(true);
           }}
           title="Show Still"
         >
           <div className="bg-black/40 hover:bg-black/60 p-3 rounded-full backdrop-blur-md transition-colors shadow-xl border border-white/10">
               <Eye className="w-6 h-6 text-white drop-shadow-lg opacity-80 hover:opacity-100 transition-opacity" />
           </div>
         </div>
      )}

      {isFallbackImage && type === 'still' && (
         <div className="absolute inset-0 flex items-center justify-center z-10 pointer-events-none">
           <span className="text-white font-bold text-3xl drop-shadow-lg tracking-wider uppercase opacity-90">
             {episodeNumber ? `EP ${episodeNumber}` : (title || "EP")}
           </span>
         </div>
      )}
    </div>
  );
};
