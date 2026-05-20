# RENDERER_proposal_hairpin_bottom_skyline.md

## Addendum v1.0: Automatic Hairpin Placement via Bottom Skyline

### Motivation

当前 hairpin（渐强/渐弱楔形标记）使用固定的 `yShift` 偏移量放置于五线谱下方（`hairpinOffsetY`，范围 -40..40，默认 -15）。这存在三个问题：

1. **无碰撞避免**：如果音符有向下符干、下波束或未来的下方位修饰符，hairpin 会与之重叠。
2. **多 hairpin 重叠**：当不同轨道在同一水平位置有不同的渐强/渐弱时，它们可能互相碰撞（VexFlow 的 `Modifier.Position.BELOW` 对多个 hairpin 不会自动垂直堆叠，它们渲染在同一行）。
3. **用户猜测**：用户必须手动调整 `hairpinOffsetY` 来适应每个乐谱的布局。

上方（top）的 skyline 算法（`TopSkyline`）已经成功地解决了同类问题：导航标记和 volta bracket 通过查询上方已占用空间来自动定位。本项目应在下方采用相同的模式。

### Design

#### BottomSkyline Class

创建 `BottomSkyline` 类，镜像 `TopSkyline`，但使用倒置的语义：

| 维度 | TopSkyline | BottomSkyline |
|------|-----------|---------------|
| 方向 | 上方（Y 值变小） | 下方（Y 值变大） |
| 初始化 | `POSITIVE_INFINITY` | `NEGATIVE_INFINITY` |
| `sample(x1,x2)` | 返回最小 Y（最高已占用点） | 返回最大 Y（最低已占用点） |
| `occupy(x1,x2,y)` | `Math.min(current, y)` | `Math.max(current, y)` |
| 回退值 | `stave.getYForTopText(1)` | `stave.getBottomLineY()` |

```typescript
class BottomSkyline {
  private readonly startX: number;
  private readonly bucketWidth: number;
  private readonly buckets: number[];
  private readonly fallbackBottom: number;

  constructor(startX: number, endX: number, fallbackBottom: number, bucketWidth = SKYLINE_BUCKET_WIDTH) {
    this.startX = startX;
    this.bucketWidth = bucketWidth;
    this.fallbackBottom = fallbackBottom;
    const bucketCount = Math.max(1, Math.ceil((endX - startX) / bucketWidth));
    this.buckets = Array.from({ length: bucketCount }, () => Number.NEGATIVE_INFINITY);
  }

  sample(x1: number, x2: number): number {
    const [start, end] = this.bucketRange(x1, x2);
    let bottom = Number.NEGATIVE_INFINITY;
    for (let i = start; i <= end; i++) {
      bottom = Math.max(bottom, this.buckets[i] ?? Number.NEGATIVE_INFINITY);
    }
    return Number.isFinite(bottom) ? bottom : this.fallbackBottom;
  }

  occupy(x1: number, x2: number, bottomY: number): void {
    const [start, end] = this.bucketRange(x1, x2);
    for (let i = start; i <= end; i++) {
      this.buckets[i] = Math.max(this.buckets[i] ?? Number.NEGATIVE_INFINITY, bottomY);
    }
  }

  private bucketRange(x1: number, x2: number): [number, number] {
    // identical to TopSkyline.bucketRange
  }
}
```

#### LayoutNote 不变

当前 `LayoutNote` 不需要修改。`aboveRefs` 仍然仅用于上方修饰符的 Top Skyline 追踪。当未来有下方修饰符时，可以添加对应的 `belowRefs`，但这超出了本提案的范围。

`occupyNoteInBottomSkyline` 仅追踪音符底部和 beam 占用的下方空间。

#### 渲染流程变更

当前 `renderSystem` 中的 hairpin 渲染流程：

```
1. 格式化并绘制音符
2. buildSystemLayoutState → 构建 TopSkyline, edgeNavs, voltaSpans
3. buildHairpinSpans → 按 hairpinOffsetY 渲染 hairpins（无碰撞避免）
4. 绘制 edgeNavs, voltaSpans 的 overlay
```

变更为：

```
1. 格式化并绘制音符
2. buildSystemLayoutState → 构建 TopSkyline + BottomSkyline, edgeNavs, voltaSpans
3. buildHairpinSpans → 查询 BottomSkyline 确定 Y 位置 → 渲染 hairpins → occupy 到 BottomSkyline
4. 绘制 edgeNavs, voltaSpans 的 overlay
```

`buildSystemLayoutState` 返回值扩展：

```typescript
type SystemLayoutState = {
  skyline: TopSkyline;
  bottomSkyline: BottomSkyline;  // NEW
  edgeNavs: PendingEdgeNav[];
  voltaSpans: PendingVoltaSpan[];
};
```

注意：保持 `skyline` 字段名不变（不重命名为 `topSkyline`），因为外部调用者不直接访问此字段。

#### Bottom Skyline 的构建

在 `buildSystemLayoutState` 中，TopSkyline 构建完成后，构建 BottomSkyline：

```typescript
const fallbackBottom = Math.max(...staves.map((stave) =>
  stave.getBottomLineY()
));
const bottomSkyline = new BottomSkyline(
  staves[0]?.getX() ?? 0,
  (staves.at(-1)?.getX() ?? 0) + (staves.at(-1)?.getWidth() ?? 0),
  fallbackBottom
);

for (let i = 0; i < layoutNotesByMeasure.length; i++) {
  for (const layoutNote of layoutNotesByMeasure[i] ?? []) {
    occupyNoteInBottomSkyline(layoutNote, bottomSkyline);
  }
}
```

`occupyNoteInBottomSkyline` 函数：

```typescript
function occupyNoteInBottomSkyline(layoutNote: LayoutNote, skyline: BottomSkyline): void {
  const note = layoutNote.note;
  const absoluteX = note.getAbsoluteX();
  const glyphWidth = note.getGlyphWidth?.() ?? 12;
  const x1 = absoluteX - glyphWidth / 2 - 2;
  const x2 = absoluteX + glyphWidth / 2 + 2;
  // 向下符干 → 使用 stem extents bottom；向上符干 → 使用 note bottom
  // 有 beam 的向下符干额外加 BEAM_THICKNESS 避免与波束线碰撞
  const beamed = !!(note.beam);
  const noteBottom = note.hasStem() && note.getStemDirection() === Stem.DOWN
    ? note.getStemExtents().bottomY + (beamed ? BEAM_THICKNESS : 0)
    : Math.max(...note.getYs());
  skyline.occupy(x1, x2, noteBottom);
}
```

#### Hairpin Y 位置的确定

在 `buildHairpinSpans`（或在调用处）中，每个 hairpin 的 Y 位置通过查询 `bottomSkyline` 来确定：

**单 hairpin 场景**（最常见）：

在 hairpin 的水平跨度上取 3 个采样点（左端、中点、右端），取 skyline 的最大值（即最下方已占用点），放置 hairpin 时保证最坏点的 clearance 满足 gap：

```
leftY   = bottomSkyline.sample(clipLeftX, clipLeftX + 1)
centerY = bottomSkyline.sample((clipLeftX + clipRightX) / 2, (clipLeftX + clipRightX) / 2 + 1)
rightY  = bottomSkyline.sample(clipRightX - 1, clipRightX)
criticalBottom = Math.max(leftY, centerY, rightY)
targetHairpinTopY = criticalBottom + SKYLINE_GAP_BELOW
```

然后 `yShift` 通过 `targetHairpinTopY - startAnchor.y` 计算（其中 `startAnchor` 来自 `getModifierStartXY(Position.BELOW, 0)`），这等效于从 VexFlow 的 BELOW 基线偏移到 skyline 确定的位置。

**多 hairpin 重叠场景**：

当两个 hairpin 在同一水平范围重叠时（例如 Track 1 的渐强和 Track 2 的渐弱），按从左到右遍历 hairpin，每放置一个后 occupy 它的空间，下一个 hairpin 的 sample 就会返回更低的 Y（已放置的 hairpin 底部），从而自动垂直堆叠。

遍历顺序：按 `startMeasureIndex` 升序，同一开始 measure 按类型排序（crescendo 先，decrescendo 后）——这个顺序不重要，只要确定性即可。

**hairpinOffsetY 语义变更**：

`hairpinOffsetY` 从"距离 VexFlow BELOW 位置的绝对偏移"变为"距离 skyline 基准的额外呼吸空间"：

- 范围改为 **0..20**（原 -40..40），默认值改为 **0**（原 -15）
- 正值：hairpin 更远离已占用空间
- 0：hairpin 紧贴 skyline 间隙下方

```typescript
const yShift = (targetHairpinTopY - startAnchor.y) + (options.hairpinOffsetY ?? 0);
```

设置文件 `useAppSettings.ts` 中的校验范围同步更新。这是一个破坏性变更，但由于项目处于 beta 阶段，无需迁移兼容。

#### 跨系统 Hairpin 延续

跨系统 hairpin（一个 hairpin 跨越多个系统的边界）当前通过 SVG clipping 实现：
- `buildHairpinSpans` 将跨系统 hairpin 拆分为每系统一个 `HairpinSpan`
- `clipLeftX`/`clipRightX` 控制可见区域
- 使用 `<clipPath>` + `StaveHairpin` 的 `leftShiftPx`/`rightShiftPx` 模拟截断

Bottom Skyline 不影响 clipping 逻辑。跨系统 hairpin 的每个 segment 独立地查询其所在系统的 Bottom Skyline，因此：
- 起始系统：hairpin 放在该系统的 skyline 下方
- 中间系统：hairpin 放在该系统的 skyline 下方（clipping 裁剪掉左侧和右侧的外延部分）
- 结束系统：同上

这不会产生任何新的复杂性问题，因为每个系统的 `buildSystemLayoutState` 是独立调用的。

唯一的变更：当前 clipping 矩形高度为硬编码 `+60`（`clipH = ... + 60`）。改用 skyline 确定 hairpin 位置后，clip 高度动态计算：

```typescript
const clipH = (targetHairpinTopY + HAIRPIN_FULL_HEIGHT) - clipY + 6;
```

### Scope

**仅影响模块**：
- `src/vexflow/renderer.ts`：添加 `BottomSkyline` class，修改 `buildSystemLayoutState`、`buildHairpinSpans`、渲染循环、clipping 矩形计算
- `src/vexflow/types.ts`：`hairpinOffsetY` 说明更新（语义变更、范围 0..20）
- `src/hooks/useAppSettings.ts`：`hairpinOffsetY` 默认值从 -15 改为 0，校验范围从 -40..40 改为 0..20

**不影响**：
- Parser / grammar
- DSL syntax
- IR / NormalizedScore types
- MusicXML export
- Settings UI structure（只改数值和含义，不改 shape）
- i18n（无需新增 key，现有 `settings.hairpinOffset` label 仍然正确）

### Hairpin Stacking for Overlapping Hairpins

当多个 hairpin 共享水平范围时，Bottom Skyline 自动处理堆叠。简化方案：

- VexFlow `StaveHairpin` 的 wedge height 为 12pt，opening 额外 ~4pt，总计 `HAIRPIN_FULL_HEIGHT = 16pt`
- 按 `startMeasureIndex` 升序遍历所有 hairpin：
  1. `baseY = bottomSkyline.sample(clipLeftX, clipRightX)` — 查询当前已占用最低点
  2. `targetHairpinTopY = baseY + SKYLINE_GAP_BELOW`
  3. 渲染 hairpin 在 `targetHairpinTopY`
  4. `bottomSkyline.occupy(clipLeftX, clipRightX, targetHairpinTopY + HAIRPIN_FULL_HEIGHT)` — 标记占用

第二个重叠 hairpin 的 `sample` 会返回第一个 hairpin 的底部，从而自动垂直堆叠，无需显式的 lane tracking 或碰撞检测。

因为所有 hairpin 不再使用 `getModifierStartXY(Position.BELOW, 0)` 作为 Y 基准，而是从 skyline 统一基线出发，不同 track 的 anchor note Y 差异不再影响 hairpin 的垂直位置。

### Acceptance Criteria

1. `npm run drummark` 编译通过，没有新的 TypeScript 错误
2. `BottomSkyline` class 单元测试：sample/occupy 正确性
3. 集成测试：包含 hairpin 的樂譜渲染后，hairpin 不与符干/音符底部重叠
4. 多 hairpin 重叠测试：两个重叠 hairpin 渲染在不同 Y 层级
5. 跨系统 hairpin 测试：延续的 hairpin 在各系统中正确放置
6. `hairpinOffsetY` 控件仍然有效（语义从"绝对偏移"变为"额外间距"）
7. smoke.test.ts 中的 hairpin 渲染测试通过（可能需要更新期望的 SVG 坐标）
8. `npm run build` 完整构建成功

---

### Review Round 1

#### Critical Issues

**1. Beam geometry is not tracked by Bottom Skyline**

The proposal's `occupyNoteInBottomSkyline` samples `note.getStemExtents().bottomY` for down-stems, but beam groups are drawn separately (line 965: `allBeams.forEach((beam) => beam.setContext(context).draw())`) after `voice.draw()`. When notes are beamed together, VexFlow draws a beam line that may extend below the lowest individual stem bottom. The skyline would not capture this beam, and a hairpin placed just below the stem bottom could collide with the beam.

Fix required: After `beam.draw()`, iterate beam groups and query their bounding box bottom (if VexFlow exposes it) to additionally occupy that space. Alternatively, add a fixed extra clearance for beamed notes.

**2. `StaveHairpin` Y positioning is angle-dependent, not uniform-shift**

`StaveHairpin` renders between `firstNote` and `lastNote`, and its top edge follows the angle between those two notes' BELOW positions. If `firstNote` is on line 1 (high) and `lastNote` is on line 3 (low), the hairpin's top edge slopes downward. A single `yShift` computed from `startAnchor.y` will shift the entire hairpin uniformly, but the *minimum clearance* between hairpin and skyline will vary along the span. In the worst case (high note at left, low note at right), the middle of the hairpin could be closer to the skyline than the endpoints suggest.

Fix: Sample the bottom skyline at multiple x-positions along the hairpin span, and compute `yShift` such that the minimum clearance across the entire span meets the gap requirement.

**3. `hairpinOffsetY` default change from -15 to 0 is a silent breaking change**

Currently `hairpinOffsetY` defaults to -15 (moving hairpin up toward the staff). The proposal suggests default 0 (relative to skyline). This means all existing scores will render with hairpins 15pt further down. Users upgrading will see their hairpins move without warning.

Fix: Either (a) keep the effective rendering position unchanged by computing an equivalent default, or (b) document this as a breaking change and migrate saved settings. The simplest approach: when skyline mode is active, interpret `hairpinOffsetY` such that a value of -15 produces the same visual position as before for a simple single-hairpin score.

#### Moderate Issues

**4. Clipping Y range may not cover skyline-placed hairpins**

Current clip Y calculation (line 972-976):
```typescript
const clipY = Math.min(span.startStave.getY(), span.endStave.getY());
const clipH = Math.max(...) - clipY + 60;
```

The `+60` is hardcoded and assumes hairpins stay within 60pt below the staff. With skyline-based placement, stacked hairpins could be placed much further down (e.g., note bottom + stem + beam + gap + hairpin1 + gap + hairpin2), potentially exceeding +60. A clipped hairpin would appear truncated.

Fix: Expand the clip rectangle to accommodate the skyline-determined Y position, or compute `clipH` dynamically from `hairpinY + hairpinHeight - clipY` instead of a fixed +60.

**5. `fallbackBottom` computation uses wrong VexFlow API**

The proposal uses `stave.getY() + stave.getHeight()` as the fallback bottom. VexFlow exposes `stave.getBottomLineY()` (found in `stave.js` line 192), which returns the Y coordinate of the bottom staff line. This is the correct semantic "bottom of staff" — conceptually symmetric to the Top Skyline's `stave.getYForTopText(1)` fallback.

Fix: Use `Math.max(...staves.map(s => s.getBottomLineY()))` for `fallbackBottom`. This ensures that even in measures with only rests, the skyline baseline is the bottom staff line, not some arbitrary computed height.

**6. `sample` fallback logic is technically correct but counterintuitive**

`Number.isFinite(NEGATIVE_INFINITY)` returns `false`, so the fallback triggers correctly. But this relies on the reader knowing that `-Infinity` is not finite. Consider adding a comment or tracking occupancy state explicitly.

#### Minor Issues

**7. Duplicated stacking logic in proposal**

Lines 238-284 contain two implementations for hairpin stacking: a manual `assignHairpinYShifts` function (lines 240-273) and a simplified "Bottom Skyline already handles it" approach (lines 276-283). The simplified approach is the correct one — skyline occupy naturally handles lane assignment without explicit tracking. The `assignHairpinYShifts` section adds confusion and should be removed or clearly marked as "superseded by simplified approach below."

**8. Rename `skyline` to `topSkyline` is unnecessary noise**

The proposal renames `SystemLayoutState.skyline` to `.topSkyline`. However, `systemLayout.skyline` is never accessed externally (only `edgeNavs` and `voltaSpans` are used at lines 1020-1021). The rename would require touching `buildEdgeNavs` and `buildVoltaSpans` parameter names and their internal `skyline.occupy()`/`skyline.sample()` calls. This is a pure cosmetic change that increases diff size.

Recommendation: Keep `skyline: TopSkyline` and add `bottomSkyline: BottomSkyline` — no rename needed.

**9. `belowRefs` on LayoutNote is dead code in this proposal**

Adding `belowRefs: SkylineRef[]` to `LayoutNote` is forward-looking but adds an empty array allocation per layout note with zero benefit in this change. Since no code populates it, the `for (const ref of layoutNote.belowRefs)` loop in `occupyNoteInBottomSkyline` never executes. This is harmless but adds noise.

Recommendation: Either defer to a future change, or add a comment that it's reserved for future below-staff modifiers.

**10. Hairpin anchor availability in multi-rest / measure-repeat measures**

The proposal does not address what happens when a hairpin spans a multi-rest or measure-repeat measure. Currently, `renderMeasureVoices` returns `layoutNotes: []` for multi-rests (line 1304) and measure-repeats (lines 1311, 1315). `buildHairpinSpans` calls `firstLayoutNoteAtOrAfter` which returns undefined for empty arrays, causing the hairpin to be silently skipped. This is existing behavior and not a regression, but the proposal should acknowledge that hairpins cannot start or end inside multi-rest measures, and that the Bottom Skyline for such regions will fall back to `getBottomLineY()`.

#### Design Strengths

- The symmetry with Top Skyline is well-chosen — same bucket resolution, same gap constant, same pattern of sample-then-occupy. This makes the code reviewable and maintainable.
- Keeping `hairpinOffsetY` as a user-facing control preserves backward compatibility of the settings UI structure.
- The scope boundary is correctly drawn: no parser, DSL, or IR changes needed.

#### STATUS: CHANGES_REQUESTED

The core concept is sound, but issues #1 (beams), #2 (angled hairpin clearance), and #3 (default value migration) must be addressed before this proposal is ready for implementation.

---

### Consolidated Changes (from Review Round 1 + Author Response)

The following modifications to the original proposal text have been agreed and applied:

1. **Beam thickness**: `occupyNoteInBottomSkyline` adds `BEAM_THICKNESS` (3pt) when a note is beamed with down-stem
2. **Angled clearance**: 3-point skyline sampling (left, center, right) guarantees minimum clearance across hairpin span
3. **`hairpinOffsetY`**: range 0..20, default 0, semantics changed to "additional breathing room"
4. **Clipping Y**: clip rect height computed dynamically from `hairpinY` instead of hardcoded +60
5. **`fallbackBottom`**: uses `stave.getBottomLineY()` not `stave.getY() + stave.getHeight()`
6. **Stacking**: simplified approach (sequential occupy) is canonical; `assignHairpinYShifts` pseudocode removed
7. **No rename**: `skyline: TopSkyline` stays; `bottomSkyline: BottomSkyline` added
8. **No `belowRefs`**: deferred to future change
9. **Multi-rest note**: explicitly acknowledged as fallback-to-bottomLineY

---

### Author Response

#### Response to #1: Beam geometry

Agreed. After `voice.draw()` and `beam.draw()`, beamed notes with DOWN stems have their stem lengths adjusted by VexFlow to reach the beam. `getStemExtents().bottomY` reflects this post-beam stem tip. However, the beam itself has thickness (~4pt for single beam) extending below the stem connection point.

Fix: In `occupyNoteInBottomSkyline`, when a note is beamed and has a DOWN stem, add a `BEAM_THICKNESS` constant (3pt) to the occupied bottom Y:

```typescript
const noteBottom = note.hasStem() && note.getStemDirection() === Stem.DOWN
  ? note.getStemExtents().bottomY + (note.beam ? BEAM_THICKNESS : 0)
  : Math.max(...note.getYs());
```

This leverages the per-note occupancy pattern without needing separate beam iteration. For drum notation, single beams are the norm, and 3pt is a sufficient buffer.

#### Response to #2: Angled hairpin clearance

Agreed. Sample the bottom skyline at 3 x-positions along the hairpin span (left edge, center, right edge), take the maximum (bottommost) as the critical clearance point:

```typescript
const samplePoints = [clipLeftX, (clipLeftX + clipRightX) / 2, clipRightX];
const criticalBottom = Math.max(...samplePoints.map(x => bottomSkyline.sample(x, x + 1)));
const targetHairpinTopY = criticalBottom + SKYLINE_GAP_BELOW;
```

Note: `getModifierStartXY(Position.BELOW, 0)` returns the note's BELOW anchor Y. The hairpin's rendered top edge follows `BELOW_anchor + yShift` at each end. Since we use a single uniform `yShift`, the hairpin's angle is preserved, and we guarantee that at the worst of the 3 sample points, the clearance >= `SKYLINE_GAP_BELOW`. The two endpoints are covered by the left and right samples; the center sample protects against a convex skyline between endpoints.

For drum notation (where most notes are on stable lines like 0, 2, 4), this is overly conservative but correct. The 3-point sample adds negligible cost.

#### Response to #3: Default value migration

Since this is a pre-release beta product, a clean break is acceptable. Proposed change:
- Range: -40..40 → **0..20**
- Default: -15 → **0**
- Label key: keep `settings.hairpinOffset` (English label can stay "Hairpin Offset" or be updated to "Hairpin Clearance")
- Semantics: additional breathing room beyond the skyline-determined gap

The old default of -15 was compensating for the absence of collision avoidance. With the skyline, 0 is the natural baseline. This change is documented but not auto-migrated for saved settings; the settings validation in `useAppSettings.ts` already clamps values to range.

#### Response to #4: Clipping Y range

Fix: Compute `clipH` dynamically instead of hardcoded +60. After skyline placement determines `hairpinY`, use:

```typescript
const hairpinFullHeight = 12 + 4; // height + opening buffer
const clipH = (hairpinY + hairpinFullHeight) - clipY + 6; // 6pt extra breathing
```

`hairpinY` is available after the skyline query. Pass it from the placement step into the rendering loop.

#### Response to #5: fallbackBottom

Fix: Use `stave.getBottomLineY()` instead of `stave.getY() + stave.getHeight()`.

#### Response to #6: sample fallback logic

Add a comment on the `Number.isFinite` check explaining that `NEGATIVE_INFINITY` is not finite, so the fallback triggers correctly.

#### Response to #7: Duplicated stacking logic

Remove the `assignHairpinYShifts` section (lines 238-273). The simplified approach (lines 275-283) is canonical: iterate hairpins in order, sample skyline, place, occupy. No explicit lane tracking needed.

#### Response to #8: Rename skyline→topSkyline

Not done. Keep `skyline: TopSkyline` in `SystemLayoutState`. Add `bottomSkyline: BottomSkyline` alongside.

#### Response to #9: belowRefs dead code

Defer `belowRefs` on `LayoutNote`. Remove from this proposal. When future below-staff modifiers are added, `belowRefs` tracking can be added at that time. The `occupyNoteInBottomSkyline` function will initially only track note bottoms and stems; modifier-occupancy will be a separate future change.

#### Response to #10: Multi-rest measures

Acknowledged. The proposal now notes that multi-rest and measure-repeat measures produce empty `layoutNotesByMeasure` entries, and the Bottom Skyline for those regions falls back to `getBottomLineY()`. Hairpins cannot start or end in multi-rest measures (existing constraint, unchanged by this proposal).

### Review Round 2

All three critical issues and all moderate issues from Review Round 1 have been addressed in the proposal text through the Consolidated Changes.

**Verification by issue:**

| Issue | Status | Evidence |
|-------|--------|----------|
| #1 Beam geometry | ✅ Fixed | `BEAM_THICKNESS` constant added to stem extents for beamed notes (line 139) |
| #2 Angled clearance | ✅ Fixed | 3-point skyline sampling at left/mid/right (lines 153-158) |
| #3 Default migration | ✅ Fixed | Range 0..20, default 0 (line 173) |
| #4 Clipping Y | ✅ Fixed | Dynamic `clipH` computation (line 200) |
| #5 fallbackBottom | ✅ Fixed | Uses `getBottomLineY()` (line 111) |
| #6 sample fallback | ✅ Acknowledged | Comment to be added at implementation time |
| #7 Duplicated stacking | ✅ Fixed | `assignHairpinYShifts` removed; simplified approach only (lines 220-231) |
| #8 Rename | ✅ Fixed | `skyline` field name preserved (line 96) |
| #9 belowRefs | ✅ Fixed | Removed from proposal; deferred to future (lines 66-70) |
| #10 Multi-rest | ✅ Acknowledged | Documented as existing constraint in Author Response |

**New concerns identified in this round:** None. The proposal is internally consistent and covers all identified edge cases.

#### STATUS: APPROVED

The proposal is ready for tasks file creation and user stamp.
### Supersession Note: 2026-05-20 VexFlow Removal

Historical review text in this proposal is preserved. The VexFlow-specific implementation path is superseded by `ARCHITECTURE_proposal_remove_vexflow.md`.

Future hairpin placement work must target `RenderScore -> LayoutScene -> thin adapter`.
