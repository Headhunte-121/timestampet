import React, { useState, useEffect } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';

interface SafeImageProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  srcPath: string;
  type: 'poster' | 'backdrop' | 'still';
  altText: string;
  className?: string;
  title?: string;
  fallbackSrcPath?: string;
  episodeNumber?: number;
}

export const SafeImage: React.FC<SafeImageProps> = ({ srcPath, type, altText, className, title, fallbackSrcPath, episodeNumber, ...rest }) => {
  const [imgSrc, setImgSrc] = useState<string | null>(null);
  const [hasError, setHasError] = useState(false);
  const [fallbackSrc, setFallbackSrc] = useState<string | null>(null);

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

    // Try to parse the source path. If it's a local path, use convertFileSrc.
    // If it's already an HTTP URL (e.g. from TMDB directly), use it as is.
    try {
      if (srcPath.startsWith('http://') || srcPath.startsWith('https://')) {
        setImgSrc(srcPath);
      } else {
        setImgSrc(convertFileSrc(srcPath));
      }
    } catch {
      setHasError(true);
    }
  }, [srcPath]);

  if (hasError || !imgSrc) {
    if (type === 'poster') {
      return (
        <div
          className={`flex items-center justify-center bg-gradient-to-tr from-[#0D0F14] to-[#1F222A] border border-[#2A2D35] text-center p-4 rounded-xl ${className || ''}`}
        >
          <span className="text-white font-bold drop-shadow-md text-sm md:text-base leading-tight break-words">
            {title || altText || 'Unknown Title'}
          </span>
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

  return (
    <img
      src={imgSrc!}
      alt={altText}
      className={className}
      onError={() => setHasError(true)}
      {...rest}
    />
  );
};
