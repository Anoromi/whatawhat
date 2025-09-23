import type { Result, UiAppError } from "@/bindings";

export function throwOnError<T>(result: Promise<Result<T, UiAppError>>) {
  return result.then((result) => {
    if (result.status === 'error') {
      throw new AppError(result.error.Other, result.error);
    }
    return result.data;
  });
}

class AppError extends Error {
  constructor(message: string, public readonly data: UiAppError) {
    super(message);
  }
}