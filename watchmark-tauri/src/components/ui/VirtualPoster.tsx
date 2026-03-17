import { useState, useEffect, useRef } from "react";

export const VirtualPoster = ({ children, className, heightClass = "aspect-[2/3]" }: any) => {
  const [isVisible, setIsVisible] = useState(false);
  const [hasLoaded, setHasLoaded] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setIsVisible(true);
            setHasLoaded(true);
          } else {
            setIsVisible(false);
          }
        });
      },
      {
        rootMargin: "600px 0px",
        threshold: 0,
      }
    );

    const currentRef = ref.current;
    if (currentRef) {
      observer.observe(currentRef);
    }

    return () => {
      if (currentRef) {
        observer.unobserve(currentRef);
      }
    };
  }, []);

  return (
    <div ref={ref} className={`${className} ${heightClass}`}>
      {isVisible ? children : (hasLoaded ? <div className="w-full h-full bg-transparent" /> : null)}
    </div>
  );
};
