import { Icon } from "./Icon";
import { useState, MouseEvent } from "react";
import { Star } from "lucide-react";
import { cn } from "../../utils/cn";

interface StarRatingProps {
  rating: number | null;
  onChange: (rating: number | null) => void;
}

export function StarRating({ rating, onChange }: StarRatingProps) {
  const [hoverRating, setHoverRating] = useState<number | null>(null);

  const calculateRating = (e: MouseEvent<HTMLDivElement>, index: number) => {
    const starNode = e.currentTarget;
    const rect = starNode.getBoundingClientRect();
    const isLeftHalf = e.clientX - rect.left < rect.width / 2;
    return index * 2 + (isLeftHalf ? 1 : 2);
  };

  const handleMouseMove = (e: MouseEvent<HTMLDivElement>, index: number) => {
    setHoverRating(calculateRating(e, index));
  };

  const handleClick = (e: MouseEvent<HTMLDivElement>, index: number) => {
    const newRating = calculateRating(e, index);
    if (newRating === rating) {
      onChange(null); // Un-click logic
    } else {
      onChange(newRating);
    }
  };

  const handleMouseLeave = () => {
    setHoverRating(null);
  };

  const currentRating = hoverRating !== null ? hoverRating : (rating ?? 0);

  return (
    <div className="flex items-center gap-1" onMouseLeave={handleMouseLeave}>
      {[0, 1, 2, 3, 4].map((index) => {
        const starValue = (index + 1) * 2;
        const isHalf = currentRating === starValue - 1;
        const isEmpty = currentRating < starValue - 1;

        return (
          <div
            key={index}
            className="relative cursor-pointer w-5 h-5"
            onMouseMove={(e) => handleMouseMove(e, index)}
            onClick={(e) => handleClick(e, index)}
          >
            {/* Background Empty Star */}
            <Star
              className={cn("absolute inset-0 w-5 h-5", rating === null && hoverRating === null ? "text-white/40" : "text-gray-500")}
              fill="none"
              strokeWidth={1.5}
            />
            {/* Foreground Fill */}
            {!isEmpty && (
              <div
                className="absolute inset-0 overflow-hidden"
                style={{ width: isHalf ? "50%" : "100%" }}
              >
                <Icon icon={Star} className="w-5 h-5 text-orange-500 fill-orange-500" strokeWidth={1.5} />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
