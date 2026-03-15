import React from 'react';
import { LucideIcon } from 'lucide-react';

interface IconProps extends React.ComponentProps<LucideIcon> {
  icon: LucideIcon;
}

export const Icon: React.FC<IconProps> = ({ icon: LucideIconComponent, ...props }) => {
  return (
    <LucideIconComponent
      strokeWidth={1.5}
      aria-hidden="true"
      {...props}
    />
  );
};
