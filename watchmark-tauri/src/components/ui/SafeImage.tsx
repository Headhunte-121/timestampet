import React, { useState, useEffect } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';

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

  if (type === 'backdrop' && (fallbackSrcPath === 'gradient' || hasError || !imgSrc)) {
    return (
      <div className={`relative bg-gradient-to-tr from-[#0D0F14] to-[#1F222A] overflow-hidden ${className || ''}`}>
        <div className="absolute inset-0 opacity-20 bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=1280&auto=format&fit=crop')] bg-cover bg-center mix-blend-overlay"></div>
      </div>
    );
  }

  if (hasError || !imgSrc) {
    if (type === 'poster') {
      const year = isDateKnown
        ? (isExactDate ? (releaseDate ? releaseDate.substring(0, 4) : '') : releaseDate)
        : 'TBD';

      return (
        <div
          className={`flex flex-col items-center justify-center bg-gradient-to-tr from-[#0D0F14] to-[#1F222A] border border-[#2A2D35] text-center p-4 rounded-xl ${className || ''}`}
        >
          <span className="text-white font-bold drop-shadow-md text-sm md:text-base leading-tight break-words">
            {title || altText || 'Unknown Title'}
          </span>
          {year && (
            <span className="text-gray-400 font-bold drop-shadow-md text-xs mt-2">
              {year}
            </span>
          )}
        </div>
      );
    } else if (type === 'still') {
      if (fallbackSrc) {
        return (
          <div className={`relative overflow-hidden bg-[#0D0F14] ${className || ''}`}>
            <img
              src={fallbackSrc}
              alt="Backdrop Fallback"
              className="absolute inset-0 w-full h-full object-cover blur-[12px] brightness-50"
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
        <div className={`relative bg-gradient-to-tr from-[#0D0F14] to-[#1F222A] overflow-hidden border border-[#2A2D35] ${className || ''}`}>
          <div className="absolute inset-0 opacity-20 bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=1280&auto=format&fit=crop')] bg-cover bg-center mix-blend-overlay"></div>
          <div className="absolute inset-0 flex items-center justify-center bg-black/60 backdrop-blur-sm z-10">
             <span className="text-white font-bold text-3xl drop-shadow-lg tracking-wider uppercase opacity-90">
               {episodeNumber ? `EP ${episodeNumber}` : (title || "EP")}
             </span>
          </div>
        </div>
      );
    } else if (type === 'backdrop') {
      // Unsplash Cinema Placeholder or Generic Dark Gradient
      return (
        <div className={`relative bg-gradient-to-tr from-[#0D0F14] to-[#1F222A] overflow-hidden ${className || ''}`}>
          {/* Subtle overlay texture or pattern can go here */}
          <div className="absolute inset-0 opacity-20 bg-[url('https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=1280&auto=format&fit=crop')] bg-cover bg-center mix-blend-overlay"></div>
        </div>
      );
    }
  }

  if (type === 'backdrop') {
    return (
      <div className={`relative overflow-hidden z-0 bg-gradient-to-tr from-[#1F222A] to-[#2A2D35] animate-pulse ${className || ''}`}>
        <motion.img
          src={imgSrc!}
          alt={altText}
          className={`absolute inset-0 w-full h-full object-cover z-10`}
          initial={{ opacity: 0 }}
          animate={{ opacity: isLoaded ? 1 : 0 }}
          transition={{ duration: 0.6, ease: "easeInOut" }}
          onLoad={() => setIsLoaded(true)}
          onError={() => setHasError(true)}
          {...rest as any}
        />
      </div>
    );
  }

  const shouldBlur = potentialSpoiler && !isCompleted && !isRevealed;
  const blurClass = shouldBlur ? 'backdrop-filter backdrop-blur-[25px]' : '';
  const transitionClass = 'transition-[filter] duration-300 ease-in';
  const filterStyle = shouldBlur ? { filter: 'blur(25px)' } : { filter: 'blur(0px)' };

  const commonClasses = `${className || ''} ${isFallbackImage ? 'brightness-75' : ''}`;
  const objectPositionClass = type === 'still' ? 'object-[center_20%]' : '';

  return (
    <div
        className={`relative w-full h-full overflow-hidden ${type === 'still' ? 'aspect-video' : ''} ${blurClass}`}
        onMouseEnter={() => setIsHovering(true)}
        onMouseLeave={() => setIsHovering(false)}
    >
      <motion.img
        src={imgSrc!}
        alt={altText}
        className={`${commonClasses} w-full h-full object-cover ${objectPositionClass} ${transitionClass}`}
        style={filterStyle}
        onError={() => setHasError(true)}
        {...rest as any}
      />
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
