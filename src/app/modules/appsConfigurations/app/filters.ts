
export const getSorterByText = (key: string) => (paramA: anyObject, paramB: anyObject) => {
  const a = String(paramA[key]).toLowerCase();
  const b = String(paramB[key]).toLowerCase();
  if (a < b) {
    return -1;
  }
  if (a > b) {
    return 1;
  }
  return 0;
};

export const getSorterByBool = (key: string) => (paramA: anyObject, paramB: anyObject) => {
  const a = Boolean(paramA[key]);
  const b = Boolean(paramB[key]);
  if (!a && b) {
    return 1;
  }
  if (a && !b) {
    return -1;
  }
  return 0;
};