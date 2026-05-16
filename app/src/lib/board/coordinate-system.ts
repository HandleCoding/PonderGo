export class CoordinateSystem {
  readonly boardSize: number;
  readonly boardPx: number;
  readonly margin: number;
  readonly cellPx: number;

  constructor(boardSize: number, boardPx: number = 570, margin: number = 25) {
    this.boardSize = boardSize;
    this.boardPx = boardPx;
    this.margin = margin;
    this.cellPx = (boardPx - 2 * margin) / (boardSize - 1);
  }

  stoneX(x: number): number {
    return this.margin + x * this.cellPx;
  }

  stoneY(y: number): number {
    return this.margin + y * this.cellPx;
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
    const x = Math.round((px - this.margin) / this.cellPx);
    const y = Math.round((py - this.margin) / this.cellPx);
    if (x < 0 || x >= this.boardSize || y < 0 || y >= this.boardSize) {
      return null;
    }
    return [x, y];
  }
}