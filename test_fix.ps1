# Test script to verify the illegal move fix
# This script tests the engine with the problematic FEN positions

Write-Host "Testing Sharp Rustic engine fix..." -ForegroundColor Green

# Test 1: Starting position
Write-Host "`nTest 1: Starting position" -ForegroundColor Yellow
$startFen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
Write-Host "FEN: $startFen"

# Test 2: Problematic position from the original issue
Write-Host "`nTest 2: Problematic position" -ForegroundColor Yellow
$problemFen = "r1bq1rk1/pp3ppp/2nbpn2/3p4/1PP5/P1NBPN2/5PPP/R1BQ1RK1 w - - 0 1"
Write-Host "FEN: $problemFen"

Write-Host "`nTo test the fix:" -ForegroundColor Cyan
Write-Host "1. Run the engine: .\target\release\rustic-sharp.exe" -ForegroundColor White
Write-Host "2. Send these commands:" -ForegroundColor White
Write-Host "   uci" -ForegroundColor Gray
Write-Host "   isready" -ForegroundColor Gray
Write-Host "   ucinewgame" -ForegroundColor Gray
Write-Host "   position fen $startFen" -ForegroundColor Gray
Write-Host "   go wtime 300000 btime 300000 movestogo 40" -ForegroundColor Gray
Write-Host "3. Check that the bestmove is NOT '0000'" -ForegroundColor White
Write-Host "4. Repeat with the problematic FEN" -ForegroundColor White

Write-Host "`nExpected result: The engine should return legal moves, not '0000'" -ForegroundColor Green 