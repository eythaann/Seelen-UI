/**
 * Calculates the optimal position for a follower element relative to a base element
 * while ensuring it stays within the viewport boundaries.
 *
 * @param base - The reference element to position against
 * @param follower - The element to be positioned
 * @param preferredPosition - Desired position relative to base ('bottom', 'top', 'left', 'right')
 * @returns Object with calculated top and left positions in pixels
 */
export function calculateElementPosition(
  base: HTMLElement,
  follower: HTMLElement,
  preferredPosition: 'bottom' | 'top' | 'left' | 'right',
): { top: number; left: number } {
  // Get bounding rectangles and viewport dimensions
  const baseRect = base.getBoundingClientRect();
  const followerRect = follower.getBoundingClientRect();
  const { innerWidth: viewportWidth, innerHeight: viewportHeight } = window;

  // Calculate initial position based on preferred placement
  let position = calculateInitialPosition(
    baseRect,
    followerRect,
    preferredPosition,
    viewportWidth,
    viewportHeight,
  );

  // Adjust position to ensure it stays within viewport boundaries
  position = adjustToViewport(position, followerRect, viewportWidth, viewportHeight);

  return position;
}

/**
 * Calculates the initial position for the follower element
 */
function calculateInitialPosition(
  baseRect: DOMRect,
  followerRect: DOMRect,
  preferredPosition: string,
  viewportWidth: number,
  viewportHeight: number,
): { top: number; left: number } {
  let top = 0;
  let left = 0;

  switch (preferredPosition) {
    case 'bottom':
      top = baseRect.bottom;
      left = baseRect.left + baseRect.width / 2 - followerRect.width / 2;
      // Flip to top if it would go below viewport
      if (top + followerRect.height > viewportHeight) {
        top = baseRect.top - followerRect.height;
      }
      break;

    case 'top':
      top = baseRect.top - followerRect.height;
      left = baseRect.left + baseRect.width / 2 - followerRect.width / 2;
      // Flip to bottom if it would go above viewport
      if (top < 0) {
        top = baseRect.bottom;
      }
      break;

    case 'left':
      left = baseRect.left - followerRect.width;
      top = baseRect.top + baseRect.height / 2 - followerRect.height / 2;
      // Flip to right if it would go beyond left viewport edge
      if (left < 0) {
        left = baseRect.right;
      }
      break;

    case 'right':
    default:
      left = baseRect.right;
      top = baseRect.top + baseRect.height / 2 - followerRect.height / 2;
      // Flip to left if it would go beyond right viewport edge
      if (left + followerRect.width > viewportWidth) {
        left = baseRect.left - followerRect.width;
      }
      break;
  }

  return { top, left };
}

/**
 * Adjusts the position to ensure the element stays within viewport boundaries
 */
function adjustToViewport(
  position: { top: number; left: number },
  followerRect: DOMRect,
  viewportWidth: number,
  viewportHeight: number,
): { top: number; left: number } {
  let { top, left } = position;

  // Horizontal boundary checks
  left = Math.max(0, left); // Can't go beyond left edge
  left = Math.min(left, viewportWidth - followerRect.width); // Can't go beyond right edge

  // Vertical boundary checks
  top = Math.max(0, top); // Can't go above top edge
  top = Math.min(top, viewportHeight - followerRect.height); // Can't go below bottom edge

  return { top, left };
}