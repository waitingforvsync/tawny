/*        00          01          02          03          04          05          06          07          08          09          0A          0B          0C          0D          0E          0F         */
/* 00 */  brk(),      lindx(ora), kil(),      mindx(slo), lzp(nop),   lzp(ora),   mzp(asl),   mzp(slo),   push(php),  imm(ora),   acc(asl),   acc(anc),   labs(nop),  labs(ora),  mabs(asl),  mabs(slo),
/* 10 */  bra(bpl),   lindy(ora), kil(),      mindy(slo), lzpx(nop),  lzpx(ora),  mzpx(asl),  mzpx(slo),  imp(clc),   labsy(ora), imp(nop),   mabsy(slo), labsx(nop), labsx(ora), mabsx(asl), mabsx(slo),
/* 20 */  jsr(),      lindx(and), kil(),      mindx(rla), lzp(bit),   lzp(and),   mzp(rol),   mzp(rla),   pull(plp),  imm(and),   acc(rol),   acc(anc),   labs(bit),  labs(and),  mabs(rol),  mabs(rla),
/* 30 */  bra(bmi),   lindy(and), kil(),      mindy(rla), lzpx(nop),  lzpx(and),  mzpx(rol),  mzpx(rla),  imp(sec),   labsy(and), imp(nop),   mabsy(rla), labsx(nop), labsx(and), mabsx(rol), mabsx(rla),
/* 40 */  rti(),      lindx(eor), kil(),      mindx(sre), lzp(nop),   lzp(eor),   mzp(lsr),   mzp(sre),   push(pha),  imm(eor),   acc(lsr),   acc(alr),   jmp(),      labs(eor),  mabs(lsr),  mabs(sre),
/* 50 */  bra(bvc),   lindy(eor), kil(),      mindy(sre), lzpx(nop),  lzpx(eor),  mzpx(lsr),  mzpx(sre),  imp(cli),   labsy(eor), imp(nop),   mabsy(sre), labsx(nop), labsx(eor), mabsx(lsr), mabsx(sre),
/* 60 */  rts(),      lindx(adc), kil(),      mindx(rra), lzp(nop),   lzp(adc),   mzp(ror),   mzp(rra),   pull(pla),  imm(adc),   acc(ror),   acc(arr),   jmpind(),   labs(adc),  mabs(ror),  mabs(rra),
/* 70 */  bra(bvs),   lindy(adc), kil(),      mindy(rra), lzpx(nop),  lzpx(adc),  mzpx(ror),  mzpx(rra),  imp(sei),   labsy(adc), imp(nop),   mabsy(rra), labsx(nop), labsx(adc), mabsx(ror), mabsx(rra),
/* 80 */  imm(nop),   sindx(sta), imm(nop),   sindx(sax), szp(sty),   szp(sta),   szp(stx),   szp(sax),   imp(dey),   imm(nop),   imp(txa),   acc(ane),   sabs(sty),  sabs(sta),  sabs(stx),  sabs(sax),
/* 90 */  bra(bcc),   sindy(sta), kil(),      sindy(sax), szpx(sty),  szpx(sta),  szpy(stx),  szpy(sax),  imp(tya),   sabsy(sta), imp(txs),   sabsy(tas), sabsx(shy), sabsx(sta), sabsy(shx), sabsy(sha),
/* A0 */  imm(ldy),   lindx(lda), imm(ldx),   lindx(lax), lzp(ldy),   lzp(lda),   lzp(ldx),   lzp(lax),   imp(tay),   imm(lda),   imp(tax),   acc(lxa),   labs(ldy),  labs(lda),  labs(ldx),  labs(lax),
/* B0 */  bra(bcs),   lindy(lda), kil(),      lindy(lax), lzpx(ldy),  lzpx(lda),  lzpy(ldx),  lzpy(lax),  imp(clv),   labsy(lda), imp(tsx),   labsy(las), labsx(ldy), labsx(lda), labsy(ldx), labsy(lax),
/* C0 */  imm(cpy),   lindx(cmp), imm(nop),   mindx(dcp), lzp(cpy),   lzp(cmp),   mzp(dec),   mzp(dcp),   imp(iny),   imm(cmp),   imp(dex),   acc(sbx),   labs(cpy),  labs(cmp),  mabs(dec),  mabs(dcp),
/* D0 */  bra(bne),   lindy(cmp), kil(),      mindy(dcp), lzpx(nop),  lzpx(cmp),  mzpx(dec),  mzpx(dcp),  imp(cld),   labsy(cmp), imp(nop),   mabsy(dcp), labsx(nop), labsx(cmp), mabsx(dec), mabsx(dcp),
/* E0 */  imm(cpx),   lindx(sbc), imm(nop),   mindx(isc), lzp(cpx),   lzp(sbc),   mzp(inc),   mzp(isc),   imp(inx),   imm(sbc),   imp(nop),   acc(sbc),   labs(cpx),  labs(sbc),  mabs(inc),  mabs(isc),
/* F0 */  bra(beq),   lindy(sbc), kil(),      mindy(isc), lzpx(nop),  lzpx(sbc),  mzpx(inc),  mzpx(isc),  imp(sed),   labsy(sbc), imp(nop),   mabsy(isc), labsx(nop), labsx(sbc), mabsx(inc), mabsx(isc)

#undef imp
#undef acc
#undef imm
#undef lzp
#undef lzpx
#undef lzpy
#undef labs
#undef labsx
#undef labsy
#undef lindx
#undef lindy
#undef szp
#undef szpx
#undef szpy
#undef sabs
#undef sabsx
#undef sabsy
#undef sindx
#undef sindy
#undef mzp
#undef mzpx
#undef mabs
#undef mabsx
#undef mabsy
#undef mindx
#undef mindy
#undef push
#undef pull
#undef bra
#undef brk
#undef jsr
#undef jmp
#undef jmpind
#undef kil
#undef rts
#undef rti
