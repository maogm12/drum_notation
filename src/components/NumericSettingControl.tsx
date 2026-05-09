import { useRef, useState, useEffect } from "preact/compat";

function clampNumber(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function stepPrecision(step: number) {
  const stepText = step.toString();
  const decimal = stepText.indexOf(".");
  return decimal >= 0 ? stepText.length - decimal - 1 : 0;
}

function normalizeSteppedValue(value: number, min: number, max: number, step: number) {
  const precision = stepPrecision(step);
  const clamped = clampNumber(value, min, max);
  return Number(clamped.toFixed(precision));
}

function inRange(value: number, min: number, max: number): boolean {
  return value >= min && value <= max;
}

export function NumericSettingControl({
  label,
  value,
  min,
  max,
  step,
  onChange,
  ariaLabelDecrease,
  ariaLabelIncrease,
  unit = "",
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  ariaLabelDecrease?: string;
  ariaLabelIncrease?: string;
  unit?: string;
}) {
  const inputMode = stepPrecision(step) > 0 ? "decimal" : "numeric";
  const inputRef = useRef<HTMLInputElement>(null);
  const [editing, setEditing] = useState(false);
  const [rawText, setRawText] = useState("");
  const lastCommitted = useRef(value);

  useEffect(() => {
    if (!editing) {
      setRawText(`${value}${unit}`);
    }
  }, [value, editing, unit]);

  const applyValue = (next: number) => {
    const clamped = normalizeSteppedValue(next, min, max, step);
    lastCommitted.current = clamped;
    onChange(clamped);
  };

  const parseInput = (text: string) => parseFloat(text.replace(unit, ""));

  const commit = () => {
    setEditing(false);
    const parsed = parseInput(rawText);
    if (!Number.isNaN(parsed)) {
      applyValue(parsed);
    } else {
      setRawText(`${lastCommitted.current}${unit}`);
    }
  };

  const isInvalid = editing && (() => {
    const stripped = rawText.replace(unit, "");
    if (stripped === "" || stripped === "-") return false;
    const parsed = parseInput(rawText);
    return !Number.isNaN(parsed) && !inRange(parsed, min, max);
  })();

  return (
    <div className="setting-row numeric-setting-row">
      <span className="setting-label-text">{label}</span>
      <div className="setting-stepper">
        <button
          className="setting-stepper-button"
          type="button"
          aria-label={ariaLabelDecrease ?? `Decrease ${label}`}
          disabled={value <= min}
          onClick={() => applyValue(value - step)}
        >
          -
        </button>
        <input
          ref={inputRef}
          className={`setting-stepper-input${isInvalid ? " setting-stepper-input-invalid" : ""}`}
          type="text"
          inputMode={inputMode}
          value={editing ? rawText : `${value}${unit}`}
          onFocus={() => {
            setEditing(true);
            setRawText(String(value));
          }}
          onBlur={commit}
          onChange={(e) => {
            const target = e.target as HTMLInputElement;
            const raw = target.value;
            setRawText(raw);
            const parsed = parseInput(raw);
            if (raw === "" || raw === "-") return;
            if (!Number.isNaN(parsed) && inRange(parsed, min, max)) {
              applyValue(parsed);
            }
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              commit();
              inputRef.current?.blur();
            }
            if (e.key === "Escape") {
              setEditing(false);
              setRawText(`${lastCommitted.current}${unit}`);
              inputRef.current?.blur();
            }
          }}
        />
        <button
          className="setting-stepper-button"
          type="button"
          aria-label={ariaLabelIncrease ?? `Increase ${label}`}
          disabled={value >= max}
          onClick={() => applyValue(value + step)}
        >
          +
        </button>
      </div>
    </div>
  );
}