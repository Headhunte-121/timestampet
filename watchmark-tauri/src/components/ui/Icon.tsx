import React from 'react';
import { LucideIcon } from 'lucide-react';

interface IconProps extends React.SVGAttributes<SVGElement> {
  icon: LucideIcon;
  size?: number | string;
  fill?: string;
  className?: string;
  strokeWidth?: number;
}

export const Icon: React.FC<IconProps> = ({
  icon: IconComponent,
  size = 24,
  fill = "none",
  className,
  strokeWidth = 1.5,
  ...props
}) => {
  return (
    <IconComponent
      size={size}
      fill={fill}
      strokeWidth={strokeWidth}
      className={className}
      aria-hidden="true"
      {...props}
    />
  );
};
