import React, { useState } from 'react'
import './index.css'

const Button = ({
  children,
  onClick,
  className,
  size = 'normal',
  danger = false,
  disabled = false,
}: {
  children?: React.ReactNode;
  onClick?: Function;
  size?: 'normal'|'small';
  danger?: boolean;
  gutter?: number;
  className?: string;
  style?: React.CSSProperties;
  disabled?: boolean
}) => {
  const [acting, setActing] = useState(false)
  return (
    <button
      className={`${className} rel-btn${(disabled || acting) ? ' disabled' : ''} size-${size} danger-${String(danger)}`}
      disabled={disabled}
      onClick={async () => {
        setActing(true)
        if (typeof onClick === 'function') await onClick()
        setActing(false)
      }}
    >
      {children}  
    </button>
  )
}

export default Button