export function formatDateForApi(date: Date): string {
  let sb: (string | number)[] = [date.getFullYear(), "-"];
  const month = date.getMonth() + 1;
  if (month < 10) {
    sb.push("0");
  }
  sb.push(month, "-");

  const dayOfMonth = date.getDate();
  if (dayOfMonth < 10) {
    sb.push("0");
  }
  sb.push(dayOfMonth);

  return sb.join("");
}
