export const getCellIndex = (row, col, width) => {
  return row * width + col;
};

export const getCellRowCol = (index, width) => {
  const col = index % width;
  const row = Math.floor(index / width);
  return [row, col];
};
