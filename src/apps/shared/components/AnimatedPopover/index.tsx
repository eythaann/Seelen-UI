import { Popover, PopoverProps } from 'antd';
import { AnimatePresence, AnimationControls, motion, Target, TargetAndTransition, VariantLabels, Variants } from 'framer-motion';
import { useEffect, useState } from 'react';

export interface PopoverAnimationProps {
  /**
     * Properties, variant label or array of variant labels to start in.
     *
     * Set to `false` to initialise with the values in `animate` (disabling the mount animation)
     *
     * ```jsx
     * // As values
     * <motion.div initial={{ opacity: 1 }} />
     *
     * // As variant
     * <motion.div initial="visible" variants={variants} />
     *
     * // Multiple variants
     * <motion.div initial={["visible", "active"]} variants={variants} />
     *
     * // As false (disable mount animation)
     * <motion.div initial={false} animate={{ opacity: 0 }} />
     * ```
     */
  initial?: boolean | Target | VariantLabels;
  /**
   * Values to animate to, variant label(s), or `AnimationControls`.
   *
   * ```jsx
   * // As values
   * <motion.div animate={{ opacity: 1 }} />
   *
   * // As variant
   * <motion.div animate="visible" variants={variants} />
   *
   * // Multiple variants
   * <motion.div animate={["visible", "active"]} variants={variants} />
   *
   * // AnimationControls
   * <motion.div animate={animation} />
   * ```
   */
  animate?: AnimationControls | TargetAndTransition | VariantLabels | boolean;
  /**
   * A target to animate to when this component is removed from the tree.
   *
   * This component **must** be the first animatable child of an `AnimatePresence` to enable this exit animation.
   *
   * This limitation exists because React doesn't allow components to defer unmounting until after
   * an animation is complete. Once this limitation is fixed, the `AnimatePresence` component will be unnecessary.
   *
   * ```jsx
   * import { AnimatePresence, motion } from 'framer-motion'
   *
   * export const MyComponent = ({ isVisible }) => {
   *   return (
   *     <AnimatePresence>
   *        {isVisible && (
   *          <motion.div
   *            initial={{ opacity: 0 }}
   *            animate={{ opacity: 1 }}
   *            exit={{ opacity: 0 }}
   *          />
   *        )}
   *     </AnimatePresence>
   *   )
   * }
   * ```
   */
  exit?: TargetAndTransition | VariantLabels;
  /**
   * Variants allow you to define animation states and organise them by name. They allow
   * you to control animations throughout a component tree by switching a single `animate` prop.
   *
   * Using `transition` options like `delayChildren` and `staggerChildren`, you can orchestrate
   * when children animations play relative to their parent.

   *
   * After passing variants to one or more `motion` component's `variants` prop, these variants
   * can be used in place of values on the `animate`, `initial`, `whileFocus`, `whileTap` and `whileHover` props.
   *
   * ```jsx
   * const variants = {
   *   active: {
   *       backgroundColor: "#f00"
   *   },
   *   inactive: {
   *     backgroundColor: "#fff",
   *     transition: { duration: 2 }
   *   }
   * }
   *
   * <motion.div variants={variants} animate="active" />
   * ```
   */
  variants?: Variants;
}

export interface AnimatedPopoverProps extends PopoverProps {
  animationTimeMs: number;
  animationDescription: PopoverAnimationProps;
}

export default function AnimatedPopover({ children, open, onOpenChange, content, animationTimeMs, animationDescription, ...popoverProps }: AnimatedPopoverProps) {
  const [openPopover, setOpenPopover] = useState(!!open);
  const [delayedOpenPopover, setDelayedOpenPopover] = useState(false);

  useEffect(() => {
    setOpenPopover(!!open);
  }, [open]);

  useEffect(() => {
    if (openPopover) {
      setDelayedOpenPopover(openPopover);

      return () => {};
    } else {
      const timeoutId = setTimeout(() => {
        setDelayedOpenPopover(openPopover);
      }, animationTimeMs);
      return () => clearTimeout(timeoutId);
    }
  }, [openPopover]);

  useEffect(() => {
    if (onOpenChange) {
      onOpenChange(delayedOpenPopover);
    }
  }, [delayedOpenPopover]);

  return (
    <Popover
      open={delayedOpenPopover}
      onOpenChange={setOpenPopover}
      {...popoverProps}
      content={
        content &&
        <AnimatePresence>
          { openPopover &&
            <motion.div
              {...animationDescription}
              transition={{ duration: animationTimeMs / 1000 }}
            >
              <>{content}</>
            </motion.div>
          }
        </AnimatePresence>
      }
    >
      {children}
    </Popover>
  );
}