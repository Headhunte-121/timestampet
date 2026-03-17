import { useRef, useEffect, useCallback } from 'react';

export function useHorizontalScroll<T extends HTMLElement>() {
  const elRef = useRef<T>(null);

  useEffect(() => {
    const el = elRef.current;
    if (el) {
      const onWheel = (e: WheelEvent) => {
        if (e.deltaY == 0) return;

        // Prevent default vertical scroll
        e.preventDefault();

        // Convert vertical delta to horizontal scroll
        el.scrollTo({
          left: el.scrollLeft + e.deltaY,
          behavior: 'smooth'
        });
      };

      // Use passive: false so we can preventDefault
      el.addEventListener('wheel', onWheel, { passive: false });
      return () => el.removeEventListener('wheel', onWheel);
    }
  }, []);

  const scrollLeft = useCallback(() => {
      if (elRef.current) {
          // Approximate width of an item, can be adjusted
          const scrollAmount = Math.max(elRef.current.clientWidth / 2, 300);
          elRef.current.scrollBy({ left: -scrollAmount, behavior: 'smooth' });
      }
  }, []);

  const scrollRight = useCallback(() => {
      if (elRef.current) {
          const scrollAmount = Math.max(elRef.current.clientWidth / 2, 300);
          elRef.current.scrollBy({ left: scrollAmount, behavior: 'smooth' });
      }
  }, []);

  return { elRef, scrollLeft, scrollRight };
}
