function showCoordinatesMargin(boardPx: number): number {
  return Math.max(boardPx * 0.024, 9);
}

export class CoordinateSystem {
  readonly boardSize: number;
  readonly boardPx: number;
  /** 左右边距（给坐标标签留空间） */
  readonly marginLeft: number;
  readonly marginRight: number;
  /** 上下边距（较小，Yzy 风格，但会自动扩展以居中） */
  readonly marginTop: number;
  readonly marginBottom: number;
  /** 实际使用的 cellPx（由水平方向决定，垂直方向通过增加边距来适配） */
  readonly cellPx: number;

  constructor(boardSize: number, boardPx: number = 570) {
    this.boardSize = boardSize;
    this.boardPx = boardPx;
    const edgeMargin = showCoordinatesMargin(boardPx);
    this.marginLeft = edgeMargin * 1.45;
    this.marginRight = edgeMargin * 1.25;

    const totalHMargin = this.marginLeft + this.marginRight;
    this.cellPx = (boardPx - totalHMargin) / (boardSize - 1);

    const gridHeight = this.cellPx * (boardSize - 1);
    const remainingV = boardPx - gridHeight;
    this.marginTop = remainingV / 2;
    this.marginBottom = remainingV / 2;
  }

  /** 统一 margin 属性（兼容旧代码，返回左边距） */
  get margin(): number { return this.marginLeft; }

  stoneX(x: number): number {
    return this.marginLeft + x * this.cellPx;
  }

  stoneY(y: number): number {
    return this.marginTop + y * this.cellPx;
  }

  stoneRadius(): number {
    return this.cellPx * 0.45;
  }

  isStarPoint(x: number, y: number): boolean {
    if (this.boardSize === 19) {
      const stars = [3, 9, 15];
      return stars.includes(x) && stars.includes(y);
    }
    if (this.boardSize === 13) {
      const stars = [3, 6, 9];
      return stars.includes(x) && stars.includes(y);
    }
    if (this.boardSize === 9) {
      const stars = [2, 4, 6];
      return stars.includes(x) && stars.includes(y);
    }
    return false;
  }

  // Convert pixel position to board coordinates
  pixelToCoord(px: number, py: number): [number, number] | null {
    const x = Math.round((px - this.marginLeft) / this.cellPx);
    const y = Math.round((py - this.marginTop) / this.cellPx);
    if (x < 0 || x >= this.boardSize || y < 0 || y >= this.boardSize) {
      return null;
    }
    return [x, y];
  }
}
