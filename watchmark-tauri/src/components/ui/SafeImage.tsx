import React, { useState, useEffect } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { Eye } from 'lucide-react';

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
    if (fallbackSrcPath) {
      try {
        if (fallbackSrcPath.startsWith('http://') || fallbackSrcPath.startsWith('https://')) {
          setFallbackSrc(fallbackSrcPath);
        } else {
          setFallbackSrc(convertFileSrc(fallbackSrcPath));
        }
      } catch {
        setFallbackSrc(null);
      }
    }
  }, [fallbackSrcPath]);

  useEffect(() => {
    setHasError(false);
    setFallbackFailed(false);
    if (!srcPath || srcPath.trim() === '') {
      setHasError(true);
      return;
    }

    let urlToRevoke: string | null = null;

    // Try to parse the source path. If it's a local path, use convertFileSrc.
    // If it's already an HTTP URL (e.g. from TMDB directly), use it as is.
    try {
      if (srcPath.startsWith('http://') || srcPath.startsWith('https://') || srcPath.startsWith('asset.localhost') || srcPath.startsWith('asset://')) {
        setImgSrc(srcPath);
      } else {
        const fileUrl = convertFileSrc(srcPath);
        setImgSrc(fileUrl);
        urlToRevoke = fileUrl;
      }
    } catch {
      setHasError(true);
    }

    // High-resolution backdrop memory eviction
    return () => {
      if (type === 'backdrop' && urlToRevoke && urlToRevoke.startsWith('asset://')) {
        setImgSrc(null);
        URL.revokeObjectURL(urlToRevoke);
      }
    };
  }, [srcPath, type]);

  const handleImageError = (e: any) => {
    try {
      e.target.style.opacity = '0';
      e.target.style.objectPosition = 'transparent';
      setHasError(true);
    } catch (err) {
      console.warn("Failed to set image error fallback state silently.");
    }
  };

  const handleFallbackError = (e: any) => {
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
    const isCinemaMode = document.documentElement.getAttribute('data-cinema-mode') !== 'false';
    const isHeroBackdrop = className?.includes('hero-backdrop-animation');

    return (
      <div className={`relative overflow-hidden z-0 ${isLoaded ? '' : 'animate-pulse'} ${className || ''}`} style={!isLoaded ? { background: 'linear-gradient(to bottom right, #1F222A, #0D0F14)' } : undefined}>
        <motion.img
          src={imgSrc!}
          alt={altText}
          className={`absolute inset-0 w-full h-full object-cover z-10 origin-center`}
          initial={{ opacity: 0, scale: 1 }}
          animate={
            isLoaded
              ? { opacity: 1, scale: isHeroBackdrop && isCinemaMode ? [1, 1.05, 1] : 1 }
              : { opacity: 0, scale: 1 }
          }
          transition={{
            opacity: { duration: 0.6, ease: "easeInOut" },
            scale: isHeroBackdrop && isCinemaMode ? { duration: 30, repeat: Infinity, ease: "linear" } : { duration: 0 }
          }}
          onLoad={() => setIsLoaded(true)}
          onError={handleImageError}
          {...rest as any}
        />
      </div>
    );
  }

  const shouldBlur = potentialSpoiler && !isCompleted && !isRevealed;
  const commonClasses = `${className || ''} ${isFallbackImage ? 'brightness-75' : ''}`;
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
        className={`relative w-full h-full overflow-hidden ${type === 'still' ? 'aspect-video' : ''}`}
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
          className={`${commonClasses} w-full h-full object-cover`}
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
