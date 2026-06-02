export function toErrorMessage(error: unknown, fallback = 'Unexpected error'): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  if (typeof error === 'string' && error.trim()) {
    return error;
  }

  try {
    const serialized = JSON.stringify(error);
    return serialized === undefined ? fallback : serialized;
  } catch {
    return fallback;
  }
}

export function logError(context: string, error: unknown): string {
  const message = toErrorMessage(error);
  console.error(`${context}:`, error);
  return message;
}
