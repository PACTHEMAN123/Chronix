
user/target/riscv64gc-unknown-none-elf/release/user_shell:	file format elf64-littleriscv

Disassembly of section .text:

0000000000010000 <_start>:
   10000: 7175         	addi	sp, sp, -0x90
   10002: e506         	sd	ra, 0x88(sp)
   10004: e122         	sd	s0, 0x80(sp)
   10006: fca6         	sd	s1, 0x78(sp)
   10008: f8ca         	sd	s2, 0x70(sp)
   1000a: f4ce         	sd	s3, 0x68(sp)
   1000c: f0d2         	sd	s4, 0x60(sp)
   1000e: ecd6         	sd	s5, 0x58(sp)
   10010: e8da         	sd	s6, 0x50(sp)
   10012: e4de         	sd	s7, 0x48(sp)
   10014: e0e2         	sd	s8, 0x40(sp)
   10016: 0900         	addi	s0, sp, 0x90
   10018: 892a         	mv	s2, a0
   1001a: 00053a03     	ld	s4, 0x0(a0)

000000000001001e <.Lpcrel_hi14>:
   1001e: 0000f517     	auipc	a0, 0xf
   10022: fe250993     	addi	s3, a0, -0x1e
   10026: 4505         	li	a0, 0x1
   10028: 00a9b4af     	amoadd.d	s1, a0, (s3)
   1002c: 0089b503     	ld	a0, 0x8(s3)
   10030: 0230000f     	fence	r, rw
   10034: 00950a63     	beq	a0, s1, 0x10048 <.Lpcrel_hi14+0x2a>
   10038: 0100000f     	fence	w, 0
   1003c: 0089b503     	ld	a0, 0x8(s3)
   10040: 0230000f     	fence	r, rw
   10044: fe951ae3     	bne	a0, s1, 0x10038 <.Lpcrel_hi14+0x1a>
   10048: 01098513     	addi	a0, s3, 0x10

000000000001004c <.Lpcrel_hi15>:
   1004c: 00007597     	auipc	a1, 0x7
   10050: fb458593     	addi	a1, a1, -0x4c
   10054: 6621         	lui	a2, 0x8
   10056: 00002097     	auipc	ra, 0x2
   1005a: 094080e7     	jalr	0x94(ra) <_ZN22buddy_system_allocator4Heap4init17h873e3cda72e1c95fE>
   1005e: 4581         	li	a1, 0x0
   10060: 0485         	addi	s1, s1, 0x1
   10062: 0310000f     	fence	rw, w
   10066: 0099b423     	sd	s1, 0x8(s3)
   1006a: f6043823     	sd	zero, -0x90(s0)
   1006e: 4521         	li	a0, 0x8
   10070: f6a43c23     	sd	a0, -0x88(s0)
   10074: f8043023     	sd	zero, -0x80(s0)
   10078: 0c0a0063     	beqz	s4, 0x10138 <.Lpcrel_hi19+0x1c>
   1007c: 4481         	li	s1, 0x0
   1007e: 00890a93     	addi	s5, s2, 0x8
   10082: f9040993     	addi	s3, s0, -0x70

0000000000010086 <.Lpcrel_hi16>:
   10086: 00005517     	auipc	a0, 0x5
   1008a: 68a50913     	addi	s2, a0, 0x68a
   1008e: a00d         	j	0x100b0 <.Lpcrel_hi16+0x2a>
   10090: f7843503     	ld	a0, -0x88(s0)
   10094: 0485         	addi	s1, s1, 0x1
   10096: 004b9593     	slli	a1, s7, 0x4
   1009a: 952e         	add	a0, a0, a1
   1009c: 01653023     	sd	s6, 0x0(a0)
   100a0: 01853423     	sd	s8, 0x8(a0)
   100a4: 001b8593     	addi	a1, s7, 0x1
   100a8: f8b43023     	sd	a1, -0x80(s0)
   100ac: 09448463     	beq	s1, s4, 0x10134 <.Lpcrel_hi19+0x18>
   100b0: 00349513     	slli	a0, s1, 0x3
   100b4: 9556         	add	a0, a0, s5
   100b6: 610c         	ld	a1, 0x0(a0)
   100b8: 567d         	li	a2, -0x1
   100ba: 00c58533     	add	a0, a1, a2
   100be: 00154503     	lbu	a0, 0x1(a0)
   100c2: 0605         	addi	a2, a2, 0x1
   100c4: f97d         	bnez	a0, 0x100ba <.Lpcrel_hi16+0x34>
   100c6: f8840513     	addi	a0, s0, -0x78
   100ca: 00004097     	auipc	ra, 0x4
   100ce: 8de080e7     	jalr	-0x722(ra) <_ZN4core3str8converts9from_utf817ha5989fec8d0859a2E>
   100d2: f8843503     	ld	a0, -0x78(s0)
   100d6: e11d         	bnez	a0, 0x100fc <.Lpcrel_hi16+0x76>
   100d8: f9043b03     	ld	s6, -0x70(s0)
   100dc: f8043b83     	ld	s7, -0x80(s0)
   100e0: f7043503     	ld	a0, -0x90(s0)
   100e4: f9843c03     	ld	s8, -0x68(s0)
   100e8: faab94e3     	bne	s7, a0, 0x10090 <.Lpcrel_hi16+0xa>
   100ec: f7040513     	addi	a0, s0, -0x90
   100f0: 85ca         	mv	a1, s2
   100f2: 00002097     	auipc	ra, 0x2
   100f6: dc8080e7     	jalr	-0x238(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE>
   100fa: bf59         	j	0x10090 <.Lpcrel_hi16+0xa>
   100fc: 0089b503     	ld	a0, 0x8(s3)
   10100: 0009b583     	ld	a1, 0x0(s3)
   10104: faa43423     	sd	a0, -0x58(s0)
   10108: fab43023     	sd	a1, -0x60(s0)

000000000001010c <.Lpcrel_hi17>:
   1010c: 00005517     	auipc	a0, 0x5
   10110: 63c50513     	addi	a0, a0, 0x63c

0000000000010114 <.Lpcrel_hi18>:
   10114: 00005597     	auipc	a1, 0x5
   10118: 61458693     	addi	a3, a1, 0x614

000000000001011c <.Lpcrel_hi19>:
   1011c: 00005597     	auipc	a1, 0x5
   10120: 65c58713     	addi	a4, a1, 0x65c
   10124: 02b00593     	li	a1, 0x2b
   10128: fa040613     	addi	a2, s0, -0x60
   1012c: 00003097     	auipc	ra, 0x3
   10130: 9c2080e7     	jalr	-0x63e(ra) <_ZN4core6result13unwrap_failed17h097de9be360fa68cE>
   10134: f7843503     	ld	a0, -0x88(s0)
   10138: 00000097     	auipc	ra, 0x0
   1013c: 40e080e7     	jalr	0x40e(ra) <main>
   10140: 2501         	sext.w	a0, a0
   10142: 00002097     	auipc	ra, 0x2
   10146: 992080e7     	jalr	-0x66e(ra) <_ZN8user_lib4exit17h98d922c79b0ff27aE>

000000000001014a <_ZN10user_shell16ProcessArguments3new17hf68a3e363dff7414E>:
   1014a: 716d         	addi	sp, sp, -0x110
   1014c: e606         	sd	ra, 0x108(sp)
   1014e: e222         	sd	s0, 0x100(sp)
   10150: fda6         	sd	s1, 0xf8(sp)
   10152: f9ca         	sd	s2, 0xf0(sp)
   10154: f5ce         	sd	s3, 0xe8(sp)
   10156: f1d2         	sd	s4, 0xe0(sp)
   10158: edd6         	sd	s5, 0xd8(sp)
   1015a: e9da         	sd	s6, 0xd0(sp)
   1015c: e5de         	sd	s7, 0xc8(sp)
   1015e: e1e2         	sd	s8, 0xc0(sp)
   10160: fd66         	sd	s9, 0xb8(sp)
   10162: f96a         	sd	s10, 0xb0(sp)
   10164: f56e         	sd	s11, 0xa8(sp)
   10166: 0a00         	addi	s0, sp, 0x110
   10168: 89aa         	mv	s3, a0
   1016a: f2043023     	sd	zero, -0xe0(s0)
   1016e: f2c43423     	sd	a2, -0xd8(s0)
   10172: f2b43823     	sd	a1, -0xd0(s0)
   10176: f2c43c23     	sd	a2, -0xc8(s0)
   1017a: f4043023     	sd	zero, -0xc0(s0)
   1017e: f4c43423     	sd	a2, -0xb8(s0)
   10182: 4b85         	li	s7, 0x1
   10184: 025b9513     	slli	a0, s7, 0x25
   10188: 02050513     	addi	a0, a0, 0x20
   1018c: f4a43823     	sd	a0, -0xb0(s0)
   10190: f5740c23     	sb	s7, -0xa8(s0)
   10194: f7741023     	sh	s7, -0xa0(s0)

0000000000010198 <.Lpcrel_hi1>:
   10198: 00005517     	auipc	a0, 0x5
   1019c: f8850493     	addi	s1, a0, -0x78
   101a0: f0840513     	addi	a0, s0, -0xf8
   101a4: f2040593     	addi	a1, s0, -0xe0
   101a8: 8626         	mv	a2, s1
   101aa: 00001097     	auipc	ra, 0x1
   101ae: 716080e7     	jalr	0x716(ra) <_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h5b91d505f4d5c4f4E>
   101b2: f1843503     	ld	a0, -0xe8(s0)
   101b6: f1043903     	ld	s2, -0xf0(s0)
   101ba: 0512         	slli	a0, a0, 0x4
   101bc: 00a90633     	add	a2, s2, a0
   101c0: f6840513     	addi	a0, s0, -0x98
   101c4: 85ca         	mv	a1, s2
   101c6: 86a6         	mv	a3, s1
   101c8: 00001097     	auipc	ra, 0x1
   101cc: 4a8080e7     	jalr	0x4a8(ra) <_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h5a536bd7b0ac802eE>
   101d0: f7843a03     	ld	s4, -0x88(s0)
   101d4: 0c0a0763     	beqz	s4, 0x102a2 <.Lpcrel_hi1+0x10a>
   101d8: f7043d83     	ld	s11, -0x90(s0)
   101dc: 4481         	li	s1, 0x0
   101de: 010d8593     	addi	a1, s11, 0x10
   101e2: 003a1513     	slli	a0, s4, 0x3
   101e6: 005a1613     	slli	a2, s4, 0x5
   101ea: 8e09         	sub	a2, a2, a0
   101ec: 4689         	li	a3, 0x2
   101ee: 03c00713     	li	a4, 0x3c
   101f2: a039         	j	0x10200 <.Lpcrel_hi1+0x68>
   101f4: 00148513     	addi	a0, s1, 0x1
   101f8: 1621         	addi	a2, a2, -0x18
   101fa: 05e1         	addi	a1, a1, 0x18
   101fc: 84aa         	mv	s1, a0
   101fe: ce41         	beqz	a2, 0x10296 <.Lpcrel_hi1+0xfe>
   10200: 6188         	ld	a0, 0x0(a1)
   10202: fed519e3     	bne	a0, a3, 0x101f4 <.Lpcrel_hi1+0x5c>
   10206: ff85b503     	ld	a0, -0x8(a1)
   1020a: 00154783     	lbu	a5, 0x1(a0)
   1020e: 00054503     	lbu	a0, 0x0(a0)
   10212: 07a2         	slli	a5, a5, 0x8
   10214: 8fc9         	or	a5, a5, a0
   10216: 00148513     	addi	a0, s1, 0x1
   1021a: fce79fe3     	bne	a5, a4, 0x101f8 <.Lpcrel_hi1+0x60>
   1021e: 31457263     	bgeu	a0, s4, 0x10522 <.Lpcrel_hi4>
   10222: 00351593     	slli	a1, a0, 0x3
   10226: 0516         	slli	a0, a0, 0x5
   10228: 8d0d         	sub	a0, a0, a1
   1022a: 00ad85b3     	add	a1, s11, a0
   1022e: f8040513     	addi	a0, s0, -0x80
   10232: 00002097     	auipc	ra, 0x2
   10236: 57c080e7     	jalr	0x57c(ra) <_ZN60_$LT$alloc..string..String$u20$as$u20$core..clone..Clone$GT$5clone17h427ba43c617bed7bE>
   1023a: 5575         	li	a0, -0x3
   1023c: 00248a93     	addi	s5, s1, 0x2
   10240: 26956f63     	bltu	a0, s1, 0x104be <.Lpcrel_hi2>
   10244: 295a6763     	bltu	s4, s5, 0x104d2 <.Lpcrel_hi3>
   10248: f8043c03     	ld	s8, -0x80(s0)
   1024c: f8843c83     	ld	s9, -0x78(s0)
   10250: 00349513     	slli	a0, s1, 0x3
   10254: 00549593     	slli	a1, s1, 0x5
   10258: 8d89         	sub	a1, a1, a0
   1025a: 00bd8b33     	add	s6, s11, a1
   1025e: 000b3583     	ld	a1, 0x0(s6)
   10262: f9043d03     	ld	s10, -0x70(s0)
   10266: f6943c23     	sd	s1, -0x88(s0)
   1026a: c981         	beqz	a1, 0x1027a <.Lpcrel_hi1+0xe2>
   1026c: 008b3503     	ld	a0, 0x8(s6)
   10270: 4605         	li	a2, 0x1
   10272: 00002097     	auipc	ra, 0x2
   10276: 896080e7     	jalr	-0x76a(ra) <__rust_dealloc>
   1027a: 018b3583     	ld	a1, 0x18(s6)
   1027e: c981         	beqz	a1, 0x1028e <.Lpcrel_hi1+0xf6>
   10280: 020b3503     	ld	a0, 0x20(s6)
   10284: 4605         	li	a2, 0x1
   10286: 00002097     	auipc	ra, 0x2
   1028a: 882080e7     	jalr	-0x77e(ra) <__rust_dealloc>
   1028e: 035a1263     	bne	s4, s5, 0x102b2 <.Lpcrel_hi1+0x11a>
   10292: 8a26         	mv	s4, s1
   10294: a0a1         	j	0x102dc <.Lpcrel_hi1+0x144>
   10296: 4d01         	li	s10, 0x0
   10298: 4b01         	li	s6, 0x0
   1029a: 4a85         	li	s5, 0x1
   1029c: 040a1463     	bnez	s4, 0x102e4 <.Lpcrel_hi1+0x14c>
   102a0: a021         	j	0x102a8 <.Lpcrel_hi1+0x110>
   102a2: 4b01         	li	s6, 0x0
   102a4: 4d01         	li	s10, 0x0
   102a6: 4a85         	li	s5, 0x1
   102a8: 4c81         	li	s9, 0x0
   102aa: 4c01         	li	s8, 0x0
   102ac: 4a01         	li	s4, 0x0
   102ae: 4521         	li	a0, 0x8
   102b0: a2a5         	j	0x10418 <.Lpcrel_hi5+0x30>
   102b2: 003a9513     	slli	a0, s5, 0x3
   102b6: 005a9593     	slli	a1, s5, 0x5
   102ba: 8d89         	sub	a1, a1, a0
   102bc: 95ee         	add	a1, a1, s11
   102be: 415a0a33     	sub	s4, s4, s5
   102c2: 003a1513     	slli	a0, s4, 0x3
   102c6: 005a1613     	slli	a2, s4, 0x5
   102ca: 8e09         	sub	a2, a2, a0
   102cc: 855a         	mv	a0, s6
   102ce: 00004097     	auipc	ra, 0x4
   102d2: 350080e7     	jalr	0x350(ra) <memmove>
   102d6: 9a26         	add	s4, s4, s1
   102d8: f7443c23     	sd	s4, -0x88(s0)
   102dc: 8b62         	mv	s6, s8
   102de: 8ae6         	mv	s5, s9
   102e0: fc0a04e3     	beqz	s4, 0x102a8 <.Lpcrel_hi1+0x110>
   102e4: 4481         	li	s1, 0x0
   102e6: 010d8593     	addi	a1, s11, 0x10
   102ea: 003a1513     	slli	a0, s4, 0x3
   102ee: 005a1613     	slli	a2, s4, 0x5
   102f2: 8e09         	sub	a2, a2, a0
   102f4: 4689         	li	a3, 0x2
   102f6: 03e00713     	li	a4, 0x3e
   102fa: a039         	j	0x10308 <.Lpcrel_hi1+0x170>
   102fc: 00148513     	addi	a0, s1, 0x1
   10300: 1621         	addi	a2, a2, -0x18
   10302: 05e1         	addi	a1, a1, 0x18
   10304: 84aa         	mv	s1, a0
   10306: c245         	beqz	a2, 0x103a6 <.Lpcrel_hi1+0x20e>
   10308: 6188         	ld	a0, 0x0(a1)
   1030a: fed519e3     	bne	a0, a3, 0x102fc <.Lpcrel_hi1+0x164>
   1030e: ff85b503     	ld	a0, -0x8(a1)
   10312: 00154783     	lbu	a5, 0x1(a0)
   10316: 00054503     	lbu	a0, 0x0(a0)
   1031a: 07a2         	slli	a5, a5, 0x8
   1031c: 8fc9         	or	a5, a5, a0
   1031e: 00148513     	addi	a0, s1, 0x1
   10322: fce79fe3     	bne	a5, a4, 0x10300 <.Lpcrel_hi1+0x168>
   10326: f1543023     	sd	s5, -0x100(s0)
   1032a: 21457563     	bgeu	a0, s4, 0x10534 <.Lpcrel_hi9>
   1032e: 00351593     	slli	a1, a0, 0x3
   10332: 0516         	slli	a0, a0, 0x5
   10334: 8d0d         	sub	a0, a0, a1
   10336: 00ad85b3     	add	a1, s11, a0
   1033a: f8040513     	addi	a0, s0, -0x80
   1033e: 00002097     	auipc	ra, 0x2
   10342: 470080e7     	jalr	0x470(ra) <_ZN60_$LT$alloc..string..String$u20$as$u20$core..clone..Clone$GT$5clone17h427ba43c617bed7bE>
   10346: 5575         	li	a0, -0x3
   10348: 00248a93     	addi	s5, s1, 0x2
   1034c: 18956d63     	bltu	a0, s1, 0x104e6 <.Lpcrel_hi7>
   10350: ef643c23     	sd	s6, -0x108(s0)
   10354: 1b5a6363     	bltu	s4, s5, 0x104fa <.Lpcrel_hi8>
   10358: f8043c03     	ld	s8, -0x80(s0)
   1035c: f8843b83     	ld	s7, -0x78(s0)
   10360: 00349513     	slli	a0, s1, 0x3
   10364: 00549593     	slli	a1, s1, 0x5
   10368: 8d89         	sub	a1, a1, a0
   1036a: 00bd8b33     	add	s6, s11, a1
   1036e: 000b3583     	ld	a1, 0x0(s6)
   10372: f9043c83     	ld	s9, -0x70(s0)
   10376: f6943c23     	sd	s1, -0x88(s0)
   1037a: c981         	beqz	a1, 0x1038a <.Lpcrel_hi1+0x1f2>
   1037c: 008b3503     	ld	a0, 0x8(s6)
   10380: 4605         	li	a2, 0x1
   10382: 00001097     	auipc	ra, 0x1
   10386: 786080e7     	jalr	0x786(ra) <__rust_dealloc>
   1038a: 018b3583     	ld	a1, 0x18(s6)
   1038e: c981         	beqz	a1, 0x1039e <.Lpcrel_hi1+0x206>
   10390: 020b3503     	ld	a0, 0x20(s6)
   10394: 4605         	li	a2, 0x1
   10396: 00001097     	auipc	ra, 0x1
   1039a: 772080e7     	jalr	0x772(ra) <__rust_dealloc>
   1039e: 015a1a63     	bne	s4, s5, 0x103b2 <.Lpcrel_hi1+0x21a>
   103a2: 8a26         	mv	s4, s1
   103a4: a825         	j	0x103dc <.Lpcrel_hi1+0x244>
   103a6: 4c01         	li	s8, 0x0
   103a8: 4c81         	li	s9, 0x0
   103aa: 4b85         	li	s7, 0x1
   103ac: 020a1e63     	bnez	s4, 0x103e8 <.Lpcrel_hi5>
   103b0: bdfd         	j	0x102ae <.Lpcrel_hi1+0x116>
   103b2: 003a9513     	slli	a0, s5, 0x3
   103b6: 005a9593     	slli	a1, s5, 0x5
   103ba: 8d89         	sub	a1, a1, a0
   103bc: 95ee         	add	a1, a1, s11
   103be: 415a0a33     	sub	s4, s4, s5
   103c2: 003a1513     	slli	a0, s4, 0x3
   103c6: 005a1613     	slli	a2, s4, 0x5
   103ca: 8e09         	sub	a2, a2, a0
   103cc: 855a         	mv	a0, s6
   103ce: 00004097     	auipc	ra, 0x4
   103d2: 250080e7     	jalr	0x250(ra) <memmove>
   103d6: 9a26         	add	s4, s4, s1
   103d8: f7443c23     	sd	s4, -0x88(s0)
   103dc: ef843b03     	ld	s6, -0x108(s0)
   103e0: f0043a83     	ld	s5, -0x100(s0)
   103e4: ec0a05e3     	beqz	s4, 0x102ae <.Lpcrel_hi1+0x116>

00000000000103e8 <.Lpcrel_hi5>:
   103e8: 0000f517     	auipc	a0, 0xf
   103ec: d4154003     	lbu	zero, -0x2bf(a0)
   103f0: 003a1493     	slli	s1, s4, 0x3
   103f4: 45a1         	li	a1, 0x8
   103f6: 8526         	mv	a0, s1
   103f8: 00001097     	auipc	ra, 0x1
   103fc: 6ec080e7     	jalr	0x6ec(ra) <__rust_alloc>
   10400: 10050763     	beqz	a0, 0x1050e <.Lpcrel_hi6>
   10404: 0da1         	addi	s11, s11, 0x8
   10406: 85aa         	mv	a1, a0
   10408: 8652         	mv	a2, s4
   1040a: 000db683     	ld	a3, 0x0(s11)
   1040e: e194         	sd	a3, 0x0(a1)
   10410: 167d         	addi	a2, a2, -0x1
   10412: 05a1         	addi	a1, a1, 0x8
   10414: 0de1         	addi	s11, s11, 0x18
   10416: fa75         	bnez	a2, 0x1040a <.Lpcrel_hi5+0x22>
   10418: f9443023     	sd	s4, -0x80(s0)
   1041c: f8a43423     	sd	a0, -0x78(s0)
   10420: f9443823     	sd	s4, -0x70(s0)

0000000000010424 <.Lpcrel_hi10>:
   10424: 00005517     	auipc	a0, 0x5
   10428: d5c50593     	addi	a1, a0, -0x2a4
   1042c: f8040513     	addi	a0, s0, -0x80
   10430: 00001097     	auipc	ra, 0x1
   10434: d78080e7     	jalr	-0x288(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E>
   10438: f8843503     	ld	a0, -0x78(s0)
   1043c: 003a1593     	slli	a1, s4, 0x3
   10440: 952e         	add	a0, a0, a1
   10442: 00053023     	sd	zero, 0x0(a0)
   10446: 0a05         	addi	s4, s4, 0x1
   10448: 0169b023     	sd	s6, 0x0(s3)
   1044c: 0159b423     	sd	s5, 0x8(s3)
   10450: 01a9b823     	sd	s10, 0x10(s3)
   10454: 0189bc23     	sd	s8, 0x18(s3)
   10458: f6843503     	ld	a0, -0x98(s0)
   1045c: f7043583     	ld	a1, -0x90(s0)
   10460: 0379b023     	sd	s7, 0x20(s3)
   10464: 0399b423     	sd	s9, 0x28(s3)
   10468: 02a9b823     	sd	a0, 0x30(s3)
   1046c: 02b9bc23     	sd	a1, 0x38(s3)
   10470: f7843503     	ld	a0, -0x88(s0)
   10474: f8043603     	ld	a2, -0x80(s0)
   10478: f8843683     	ld	a3, -0x78(s0)
   1047c: f0843583     	ld	a1, -0xf8(s0)
   10480: 04a9b023     	sd	a0, 0x40(s3)
   10484: 0549bc23     	sd	s4, 0x58(s3)
   10488: 04c9b423     	sd	a2, 0x48(s3)
   1048c: 04d9b823     	sd	a3, 0x50(s3)
   10490: c981         	beqz	a1, 0x104a0 <.Lpcrel_hi10+0x7c>
   10492: 0592         	slli	a1, a1, 0x4
   10494: 4621         	li	a2, 0x8
   10496: 854a         	mv	a0, s2
   10498: 00001097     	auipc	ra, 0x1
   1049c: 670080e7     	jalr	0x670(ra) <__rust_dealloc>
   104a0: 60b2         	ld	ra, 0x108(sp)
   104a2: 6412         	ld	s0, 0x100(sp)
   104a4: 74ee         	ld	s1, 0xf8(sp)
   104a6: 794e         	ld	s2, 0xf0(sp)
   104a8: 79ae         	ld	s3, 0xe8(sp)
   104aa: 7a0e         	ld	s4, 0xe0(sp)
   104ac: 6aee         	ld	s5, 0xd8(sp)
   104ae: 6b4e         	ld	s6, 0xd0(sp)
   104b0: 6bae         	ld	s7, 0xc8(sp)
   104b2: 6c0e         	ld	s8, 0xc0(sp)
   104b4: 7cea         	ld	s9, 0xb8(sp)
   104b6: 7d4a         	ld	s10, 0xb0(sp)
   104b8: 7daa         	ld	s11, 0xa8(sp)
   104ba: 6151         	addi	sp, sp, 0x110
   104bc: 8082         	ret

00000000000104be <.Lpcrel_hi2>:
   104be: 00005517     	auipc	a0, 0x5
   104c2: 10a50613     	addi	a2, a0, 0x10a
   104c6: 8526         	mv	a0, s1
   104c8: 85d6         	mv	a1, s5
   104ca: 00003097     	auipc	ra, 0x3
   104ce: 4ce080e7     	jalr	0x4ce(ra) <_ZN4core5slice5index22slice_index_order_fail17h37263f3371ee24c8E>

00000000000104d2 <.Lpcrel_hi3>:
   104d2: 00005517     	auipc	a0, 0x5
   104d6: 0f650613     	addi	a2, a0, 0xf6
   104da: 8556         	mv	a0, s5
   104dc: 85d2         	mv	a1, s4
   104de: 00003097     	auipc	ra, 0x3
   104e2: 4aa080e7     	jalr	0x4aa(ra) <_ZN4core5slice5index24slice_end_index_len_fail17h6d0be8bee959f757E>

00000000000104e6 <.Lpcrel_hi7>:
   104e6: 00005517     	auipc	a0, 0x5
   104ea: 0e250613     	addi	a2, a0, 0xe2
   104ee: 8526         	mv	a0, s1
   104f0: 85d6         	mv	a1, s5
   104f2: 00003097     	auipc	ra, 0x3
   104f6: 4a6080e7     	jalr	0x4a6(ra) <_ZN4core5slice5index22slice_index_order_fail17h37263f3371ee24c8E>

00000000000104fa <.Lpcrel_hi8>:
   104fa: 00005517     	auipc	a0, 0x5
   104fe: 0ce50613     	addi	a2, a0, 0xce
   10502: 8556         	mv	a0, s5
   10504: 85d2         	mv	a1, s4
   10506: 00003097     	auipc	ra, 0x3
   1050a: 482080e7     	jalr	0x482(ra) <_ZN4core5slice5index24slice_end_index_len_fail17h6d0be8bee959f757E>

000000000001050e <.Lpcrel_hi6>:
   1050e: 00005517     	auipc	a0, 0x5
   10512: c1250613     	addi	a2, a0, -0x3ee
   10516: 4521         	li	a0, 0x8
   10518: 85a6         	mv	a1, s1
   1051a: 00002097     	auipc	ra, 0x2
   1051e: 262080e7     	jalr	0x262(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

0000000000010522 <.Lpcrel_hi4>:
   10522: 00005597     	auipc	a1, 0x5
   10526: c2e58613     	addi	a2, a1, -0x3d2
   1052a: 85d2         	mv	a1, s4
   1052c: 00002097     	auipc	ra, 0x2
   10530: 468080e7     	jalr	0x468(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

0000000000010534 <.Lpcrel_hi9>:
   10534: 00005597     	auipc	a1, 0x5
   10538: c3458613     	addi	a2, a1, -0x3cc
   1053c: 85d2         	mv	a1, s4
   1053e: 00002097     	auipc	ra, 0x2
   10542: 456080e7     	jalr	0x456(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

0000000000010546 <main>:
   10546: dd010113     	addi	sp, sp, -0x230
   1054a: 22113423     	sd	ra, 0x228(sp)
   1054e: 22813023     	sd	s0, 0x220(sp)
   10552: 20913c23     	sd	s1, 0x218(sp)
   10556: 21213823     	sd	s2, 0x210(sp)
   1055a: 21313423     	sd	s3, 0x208(sp)
   1055e: 21413023     	sd	s4, 0x200(sp)
   10562: ffd6         	sd	s5, 0x1f8(sp)
   10564: fbda         	sd	s6, 0x1f0(sp)
   10566: f7de         	sd	s7, 0x1e8(sp)
   10568: f3e2         	sd	s8, 0x1e0(sp)
   1056a: efe6         	sd	s9, 0x1d8(sp)
   1056c: ebea         	sd	s10, 0x1d0(sp)
   1056e: e7ee         	sd	s11, 0x1c8(sp)
   10570: 1c00         	addi	s0, sp, 0x230

0000000000010572 <.Lpcrel_hi11>:
   10572: 00005517     	auipc	a0, 0x5
   10576: c7650513     	addi	a0, a0, -0x38a
   1057a: f2a43c23     	sd	a0, -0xc8(s0)
   1057e: 4b05         	li	s6, 0x1
   10580: f5643023     	sd	s6, -0xc0(s0)
   10584: f4043c23     	sd	zero, -0xa8(s0)
   10588: 49a1         	li	s3, 0x8
   1058a: f5343423     	sd	s3, -0xb8(s0)
   1058e: f4043823     	sd	zero, -0xb0(s0)
   10592: f3840513     	addi	a0, s0, -0xc8
   10596: 00002097     	auipc	ra, 0x2
   1059a: a3e080e7     	jalr	-0x5c2(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   1059e: e2043c23     	sd	zero, -0x1c8(s0)
   105a2: e5643023     	sd	s6, -0x1c0(s0)
   105a6: e4043423     	sd	zero, -0x1b8(s0)

00000000000105aa <.Lpcrel_hi12>:
   105aa: 00005517     	auipc	a0, 0x5
   105ae: d5e50513     	addi	a0, a0, -0x2a2
   105b2: e2a43423     	sd	a0, -0x1d8(s0)
   105b6: eea43823     	sd	a0, -0x110(s0)

00000000000105ba <.Lpcrel_hi13>:
   105ba: 00001517     	auipc	a0, 0x1
   105be: af850513     	addi	a0, a0, -0x508
   105c2: e2a43023     	sd	a0, -0x1e0(s0)
   105c6: eea43c23     	sd	a0, -0x108(s0)

00000000000105ca <.Lpcrel_hi14>:
   105ca: 00005517     	auipc	a0, 0x5
   105ce: bee50c13     	addi	s8, a0, -0x412
   105d2: f3843c23     	sd	s8, -0xc8(s0)
   105d6: f5643023     	sd	s6, -0xc0(s0)
   105da: f4043c23     	sd	zero, -0xa8(s0)
   105de: ef040a13     	addi	s4, s0, -0x110
   105e2: f5443423     	sd	s4, -0xb8(s0)
   105e6: f5643823     	sd	s6, -0xb0(s0)
   105ea: f3840513     	addi	a0, s0, -0xc8
   105ee: 00002097     	auipc	ra, 0x2
   105f2: 9e6080e7     	jalr	-0x61a(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   105f6: 4ab1         	li	s5, 0xc

00000000000105f8 <.Lpcrel_hi15>:
   105f8: 00005517     	auipc	a0, 0x5
   105fc: e8450513     	addi	a0, a0, -0x17c
   10600: e0a43823     	sd	a0, -0x1f0(s0)

0000000000010604 <.Lpcrel_hi16>:
   10604: 00003517     	auipc	a0, 0x3
   10608: 1e450d93     	addi	s11, a0, 0x1e4

000000000001060c <.Lpcrel_hi17>:
   1060c: 00005517     	auipc	a0, 0x5
   10610: ce450513     	addi	a0, a0, -0x31c
   10614: e0a43423     	sd	a0, -0x1f8(s0)
   10618: 4c89         	li	s9, 0x2

000000000001061a <.Lpcrel_hi18>:
   1061a: 00005517     	auipc	a0, 0x5
   1061e: be650513     	addi	a0, a0, -0x41a
   10622: e2a43823     	sd	a0, -0x1d0(s0)
   10626: 457d         	li	a0, 0x1f
   10628: 150a         	slli	a0, a0, 0x22
   1062a: 07c50513     	addi	a0, a0, 0x7c
   1062e: e0a43023     	sd	a0, -0x200(s0)

0000000000010632 <.Lpcrel_hi19>:
   10632: 00005517     	auipc	a0, 0x5
   10636: aee50513     	addi	a0, a0, -0x512
   1063a: dea43c23     	sd	a0, -0x208(s0)
   1063e: 5545         	li	a0, -0xf
   10640: 8105         	srli	a0, a0, 0x1
   10642: dea43823     	sd	a0, -0x210(s0)
   10646: 5bf9         	li	s7, -0x2
   10648: 4d35         	li	s10, 0xd
   1064a: df843423     	sd	s8, -0x218(s0)
   1064e: a83d         	j	0x1068c <.Lpcrel_hi19+0x5a>
   10650: ef040a13     	addi	s4, s0, -0x110
   10654: 4ab1         	li	s5, 0xc
   10656: de843c03     	ld	s8, -0x218(s0)
   1065a: 4d35         	li	s10, 0xd
   1065c: e2843503     	ld	a0, -0x1d8(s0)
   10660: eea43823     	sd	a0, -0x110(s0)
   10664: e2043503     	ld	a0, -0x1e0(s0)
   10668: eea43c23     	sd	a0, -0x108(s0)
   1066c: f3843c23     	sd	s8, -0xc8(s0)
   10670: f5643023     	sd	s6, -0xc0(s0)
   10674: f4043c23     	sd	zero, -0xa8(s0)
   10678: f5443423     	sd	s4, -0xb8(s0)
   1067c: f5643823     	sd	s6, -0xb0(s0)
   10680: f3840513     	addi	a0, s0, -0xc8
   10684: 00002097     	auipc	ra, 0x2
   10688: 950080e7     	jalr	-0x6b0(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   1068c: f2040c23     	sb	zero, -0xc8(s0)
   10690: f3840593     	addi	a1, s0, -0xc8
   10694: 4605         	li	a2, 0x1
   10696: 03f00893     	li	a7, 0x3f
   1069a: 4501         	li	a0, 0x0
   1069c: 00000073     	ecall
   106a0: f3840483     	lb	s1, -0xc8(s0)
   106a4: 0ff4f513     	andi	a0, s1, 0xff
   106a8: 00aac863     	blt	s5, a0, 0x106b8 <.Lpcrel_hi19+0x86>
   106ac: 01350c63     	beq	a0, s3, 0x106c4 <.Lpcrel_hi19+0x92>
   106b0: 45a9         	li	a1, 0xa
   106b2: 0cb50b63     	beq	a0, a1, 0x10788 <.Lpcrel_hi19+0x156>
   106b6: a409         	j	0x108b8 <.Lpcrel_hi21+0x94>
   106b8: 0da50863     	beq	a0, s10, 0x10788 <.Lpcrel_hi19+0x156>
   106bc: 07f00593     	li	a1, 0x7f
   106c0: 1eb51c63     	bne	a0, a1, 0x108b8 <.Lpcrel_hi21+0x94>
   106c4: e4843503     	ld	a0, -0x1b8(s0)
   106c8: d171         	beqz	a0, 0x1068c <.Lpcrel_hi19+0x5a>
   106ca: e1043483     	ld	s1, -0x1f0(s0)
   106ce: ee943823     	sd	s1, -0x110(s0)
   106d2: efb43c23     	sd	s11, -0x108(s0)
   106d6: f3843c23     	sd	s8, -0xc8(s0)
   106da: f5643023     	sd	s6, -0xc0(s0)
   106de: f4043c23     	sd	zero, -0xa8(s0)
   106e2: f5443423     	sd	s4, -0xb8(s0)
   106e6: f5643823     	sd	s6, -0xb0(s0)
   106ea: f3840513     	addi	a0, s0, -0xc8
   106ee: 00002097     	auipc	ra, 0x2
   106f2: 8e6080e7     	jalr	-0x71a(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   106f6: e0843503     	ld	a0, -0x1f8(s0)
   106fa: f2a43c23     	sd	a0, -0xc8(s0)
   106fe: f5643023     	sd	s6, -0xc0(s0)
   10702: f4043c23     	sd	zero, -0xa8(s0)
   10706: f5343423     	sd	s3, -0xb8(s0)
   1070a: f4043823     	sd	zero, -0xb0(s0)
   1070e: f3840513     	addi	a0, s0, -0xc8
   10712: 00002097     	auipc	ra, 0x2
   10716: 8c2080e7     	jalr	-0x73e(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   1071a: ee943823     	sd	s1, -0x110(s0)
   1071e: efb43c23     	sd	s11, -0x108(s0)
   10722: f3843c23     	sd	s8, -0xc8(s0)
   10726: f5643023     	sd	s6, -0xc0(s0)
   1072a: f4043c23     	sd	zero, -0xa8(s0)
   1072e: f5443423     	sd	s4, -0xb8(s0)
   10732: f5643823     	sd	s6, -0xb0(s0)
   10736: f3840513     	addi	a0, s0, -0xc8
   1073a: 00002097     	auipc	ra, 0x2
   1073e: 89a080e7     	jalr	-0x766(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   10742: e4843503     	ld	a0, -0x1b8(s0)
   10746: d139         	beqz	a0, 0x1068c <.Lpcrel_hi19+0x5a>
   10748: e4043683     	ld	a3, -0x1c0(s0)
   1074c: 96aa         	add	a3, a3, a0
   1074e: fff68603     	lb	a2, -0x1(a3)
   10752: 55fd         	li	a1, -0x1
   10754: 42065263     	bgez	a2, 0x10b78 <.Lpcrel_hi45+0x14e>
   10758: ffe6c703     	lbu	a4, -0x2(a3)
   1075c: 03871613     	slli	a2, a4, 0x38
   10760: 9661         	srai	a2, a2, 0x38
   10762: fbf00493     	li	s1, -0x41
   10766: 3ec4c463     	blt	s1, a2, 0x10b4e <.Lpcrel_hi45+0x124>
   1076a: ffd6c783     	lbu	a5, -0x3(a3)
   1076e: 03879713     	slli	a4, a5, 0x38
   10772: 9761         	srai	a4, a4, 0x38
   10774: 3ee4c263     	blt	s1, a4, 0x10b58 <.Lpcrel_hi45+0x12e>
   10778: ffc6c683     	lbu	a3, -0x4(a3)
   1077c: 8a9d         	andi	a3, a3, 0x7
   1077e: 069a         	slli	a3, a3, 0x6
   10780: 03f77713     	andi	a4, a4, 0x3f
   10784: 8ed9         	or	a3, a3, a4
   10786: aed9         	j	0x10b5c <.Lpcrel_hi45+0x132>
   10788: e3043503     	ld	a0, -0x1d0(s0)
   1078c: f2a43c23     	sd	a0, -0xc8(s0)
   10790: f5643023     	sd	s6, -0xc0(s0)
   10794: f4043c23     	sd	zero, -0xa8(s0)
   10798: f5343423     	sd	s3, -0xb8(s0)
   1079c: f4043823     	sd	zero, -0xb0(s0)
   107a0: f3840513     	addi	a0, s0, -0xc8
   107a4: 00002097     	auipc	ra, 0x2
   107a8: 830080e7     	jalr	-0x7d0(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   107ac: e4843503     	ld	a0, -0x1b8(s0)
   107b0: ea0506e3     	beqz	a0, 0x1065c <.Lpcrel_hi19+0x2a>
   107b4: 4ca1         	li	s9, 0x8
   107b6: e4043583     	ld	a1, -0x1c0(s0)
   107ba: e6043423     	sd	zero, -0x198(s0)
   107be: e6a43823     	sd	a0, -0x190(s0)
   107c2: e6b43c23     	sd	a1, -0x188(s0)
   107c6: e8a43023     	sd	a0, -0x180(s0)
   107ca: e8043423     	sd	zero, -0x178(s0)
   107ce: e8a43823     	sd	a0, -0x170(s0)
   107d2: e0043503     	ld	a0, -0x200(s0)
   107d6: e8a43c23     	sd	a0, -0x168(s0)
   107da: eb640023     	sb	s6, -0x160(s0)
   107de: eb641423     	sh	s6, -0x158(s0)
   107e2: e5040513     	addi	a0, s0, -0x1b0
   107e6: e6840593     	addi	a1, s0, -0x198
   107ea: df843603     	ld	a2, -0x208(s0)
   107ee: 00001097     	auipc	ra, 0x1
   107f2: 0d2080e7     	jalr	0xd2(ra) <_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h5b91d505f4d5c4f4E>
   107f6: e6043983     	ld	s3, -0x1a0(s0)
   107fa: 4481         	li	s1, 0x0
   107fc: 00599513     	slli	a0, s3, 0x5
   10800: 00799593     	slli	a1, s3, 0x7
   10804: 06000613     	li	a2, 0x60
   10808: 02c9b633     	mulhu	a2, s3, a2
   1080c: 40a58ab3     	sub	s5, a1, a0
   10810: 00061be3     	bnez	a2, 0x11026 <.Lpcrel_hi22>
   10814: df043503     	ld	a0, -0x210(s0)
   10818: 015567e3     	bltu	a0, s5, 0x11026 <.Lpcrel_hi22>
   1081c: e5843d03     	ld	s10, -0x1a8(s0)
   10820: 0e0a8f63     	beqz	s5, 0x1091e <.Lpcrel_hi20+0x26>

0000000000010824 <.Lpcrel_hi21>:
   10824: 0000f517     	auipc	a0, 0xf
   10828: 90554003     	lbu	zero, -0x6fb(a0)
   1082c: 45a1         	li	a1, 0x8
   1082e: 44a1         	li	s1, 0x8
   10830: 8556         	mv	a0, s5
   10832: 00001097     	auipc	ra, 0x1
   10836: 2b2080e7     	jalr	0x2b2(ra) <__rust_alloc>
   1083a: 7e050663     	beqz	a0, 0x11026 <.Lpcrel_hi22>
   1083e: 8a2a         	mv	s4, a0
   10840: 894e         	mv	s2, s3
   10842: 0e098263     	beqz	s3, 0x10926 <.Lpcrel_hi20+0x2e>
   10846: e1243c23     	sd	s2, -0x1e8(s0)
   1084a: 008d0493     	addi	s1, s10, 0x8
   1084e: 8952         	mv	s2, s4
   10850: 8c4e         	mv	s8, s3
   10852: ff84b583     	ld	a1, -0x8(s1)
   10856: 6090         	ld	a2, 0x0(s1)
   10858: f3840513     	addi	a0, s0, -0xc8
   1085c: 00000097     	auipc	ra, 0x0
   10860: 8ee080e7     	jalr	-0x712(ra) <_ZN10user_shell16ProcessArguments3new17hf68a3e363dff7414E>
   10864: f3840593     	addi	a1, s0, -0xc8
   10868: 06000613     	li	a2, 0x60
   1086c: 854a         	mv	a0, s2
   1086e: 00004097     	auipc	ra, 0x4
   10872: caa080e7     	jalr	-0x356(ra) <memcpy>
   10876: 1c7d         	addi	s8, s8, -0x1
   10878: 06090913     	addi	s2, s2, 0x60
   1087c: 04c1         	addi	s1, s1, 0x10
   1087e: fc0c1ae3     	bnez	s8, 0x10852 <.Lpcrel_hi21+0x2e>
   10882: e1843903     	ld	s2, -0x1e8(s0)
   10886: eb243823     	sd	s2, -0x150(s0)
   1088a: eb443c23     	sd	s4, -0x148(s0)
   1088e: fff98713     	addi	a4, s3, -0x1
   10892: ed343023     	sd	s3, -0x140(s0)
   10896: c375         	beqz	a4, 0x1097a <.Lpcrel_hi20+0x82>
   10898: 028a3503     	ld	a0, 0x28(s4)
   1089c: 00153613     	seqz	a2, a0
   108a0: 070a0513     	addi	a0, s4, 0x70
   108a4: 19f9         	addi	s3, s3, -0x2
   108a6: f40a8593     	addi	a1, s5, -0xc0
   108aa: 2e099f63     	bnez	s3, 0x10ba8 <.Lpcrel_hi45+0x17e>
   108ae: 6114         	ld	a3, 0x0(a0)
   108b0: 0016b693     	seqz	a3, a3
   108b4: 8e75         	and	a2, a2, a3
   108b6: a4cd         	j	0x10b98 <.Lpcrel_hi45+0x16e>
   108b8: eca42423     	sw	a0, -0x138(s0)
   108bc: ec840513     	addi	a0, s0, -0x138
   108c0: eea43823     	sd	a0, -0x110(s0)
   108c4: efb43c23     	sd	s11, -0x108(s0)
   108c8: f3843c23     	sd	s8, -0xc8(s0)
   108cc: f5643023     	sd	s6, -0xc0(s0)
   108d0: f4043c23     	sd	zero, -0xa8(s0)
   108d4: f5443423     	sd	s4, -0xb8(s0)
   108d8: f5643823     	sd	s6, -0xb0(s0)
   108dc: f3840513     	addi	a0, s0, -0xc8
   108e0: 00001097     	auipc	ra, 0x1
   108e4: 6f4080e7     	jalr	0x6f4(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   108e8: 0404ce63     	bltz	s1, 0x10944 <.Lpcrel_hi20+0x4c>
   108ec: e4843903     	ld	s2, -0x1b8(s0)
   108f0: e3843503     	ld	a0, -0x1c8(s0)
   108f4: 00a91c63     	bne	s2, a0, 0x1090c <.Lpcrel_hi20+0x14>

00000000000108f8 <.Lpcrel_hi20>:
   108f8: 00004517     	auipc	a0, 0x4
   108fc: 78850593     	addi	a1, a0, 0x788
   10900: e3840513     	addi	a0, s0, -0x1c8
   10904: 00002097     	auipc	ra, 0x2
   10908: d56080e7     	jalr	-0x2aa(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE>
   1090c: e4043503     	ld	a0, -0x1c0(s0)
   10910: 954a         	add	a0, a0, s2
   10912: 00950023     	sb	s1, 0x0(a0)
   10916: 0905         	addi	s2, s2, 0x1
   10918: e5243423     	sd	s2, -0x1b8(s0)
   1091c: bb85         	j	0x1068c <.Lpcrel_hi19+0x5a>
   1091e: 4901         	li	s2, 0x0
   10920: 4a21         	li	s4, 0x8
   10922: f20992e3     	bnez	s3, 0x10846 <.Lpcrel_hi21+0x22>
   10926: eb243823     	sd	s2, -0x150(s0)
   1092a: eb443c23     	sd	s4, -0x148(s0)
   1092e: ec043023     	sd	zero, -0x140(s0)
   10932: ec043423     	sd	zero, -0x138(s0)
   10936: 49a1         	li	s3, 0x8
   10938: ed343823     	sd	s3, -0x130(s0)
   1093c: ec043c23     	sd	zero, -0x128(s0)
   10940: 4c89         	li	s9, 0x2
   10942: aa55         	j	0x10af6 <.Lpcrel_hi45+0xcc>
   10944: e4843583     	ld	a1, -0x1b8(s0)
   10948: e3843503     	ld	a0, -0x1c8(s0)
   1094c: 8d0d         	sub	a0, a0, a1
   1094e: 22ab7963     	bgeu	s6, a0, 0x10b80 <.Lpcrel_hi45+0x156>
   10952: 03849513     	slli	a0, s1, 0x38
   10956: e4043603     	ld	a2, -0x1c0(s0)
   1095a: 9179         	srli	a0, a0, 0x3e
   1095c: fc050513     	addi	a0, a0, -0x40
   10960: 0bf4f693     	andi	a3, s1, 0xbf
   10964: 95b2         	add	a1, a1, a2
   10966: 00d580a3     	sb	a3, 0x1(a1)
   1096a: 00a58023     	sb	a0, 0x0(a1)
   1096e: e4843503     	ld	a0, -0x1b8(s0)
   10972: 0509         	addi	a0, a0, 0x2
   10974: e4a43423     	sd	a0, -0x1b8(s0)
   10978: bb11         	j	0x1068c <.Lpcrel_hi19+0x5a>
   1097a: dce43c23     	sd	a4, -0x228(s0)
   1097e: ec043423     	sd	zero, -0x138(s0)
   10982: ed943823     	sd	s9, -0x130(s0)
   10986: ec043c23     	sd	zero, -0x128(s0)
   1098a: 4c89         	li	s9, 0x2
   1098c: dfa43023     	sd	s10, -0x220(s0)
   10990: 4981         	li	s3, 0x0
   10992: 4901         	li	s2, 0x0
   10994: ee043823     	sd	zero, -0x110(s0)
   10998: 4d21         	li	s10, 0x8
   1099a: efa43c23     	sd	s10, -0x108(s0)
   1099e: f0043023     	sd	zero, -0x100(s0)
   109a2: a00d         	j	0x109c4 <.Lpcrel_hi20+0xcc>
   109a4: ef843503     	ld	a0, -0x108(s0)
   109a8: 003c1593     	slli	a1, s8, 0x3
   109ac: 952e         	add	a0, a0, a1
   109ae: e104         	sd	s1, 0x0(a0)
   109b0: 001c0513     	addi	a0, s8, 0x1
   109b4: f0a43023     	sd	a0, -0x100(s0)
   109b8: 06090913     	addi	s2, s2, 0x60
   109bc: 19fd         	addi	s3, s3, -0x1
   109be: 0d41         	addi	s10, s10, 0x10
   109c0: 092a8063     	beq	s5, s2, 0x10a40 <.Lpcrel_hi45+0x16>
   109c4: 0dc00893     	li	a7, 0xdc
   109c8: 4501         	li	a0, 0x0
   109ca: 4581         	li	a1, 0x0
   109cc: 4601         	li	a2, 0x0
   109ce: 00000073     	ecall
   109d2: f0a43423     	sd	a0, -0xf8(s0)
   109d6: f0840513     	addi	a0, s0, -0xf8
   109da: f2a43023     	sd	a0, -0xe0(s0)

00000000000109de <.Lpcrel_hi24>:
   109de: 00004517     	auipc	a0, 0x4
   109e2: 8b850513     	addi	a0, a0, -0x748
   109e6: f2a43423     	sd	a0, -0xd8(s0)

00000000000109ea <.Lpcrel_hi25>:
   109ea: 00005517     	auipc	a0, 0x5
   109ee: 87e50513     	addi	a0, a0, -0x782
   109f2: f2a43c23     	sd	a0, -0xc8(s0)
   109f6: f5943023     	sd	s9, -0xc0(s0)
   109fa: f4043c23     	sd	zero, -0xa8(s0)
   109fe: f2040513     	addi	a0, s0, -0xe0
   10a02: f4a43423     	sd	a0, -0xb8(s0)
   10a06: f5643823     	sd	s6, -0xb0(s0)
   10a0a: f3840513     	addi	a0, s0, -0xc8
   10a0e: 00001097     	auipc	ra, 0x1
   10a12: 5c6080e7     	jalr	0x5c6(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   10a16: f0843483     	ld	s1, -0xf8(s0)
   10a1a: 26048a63     	beqz	s1, 0x10c8e <.Lpcrel_hi23+0x42>
   10a1e: f0043c03     	ld	s8, -0x100(s0)
   10a22: ef043503     	ld	a0, -0x110(s0)
   10a26: f6ac1fe3     	bne	s8, a0, 0x109a4 <.Lpcrel_hi20+0xac>

0000000000010a2a <.Lpcrel_hi45>:
   10a2a: 00005517     	auipc	a0, 0x5
   10a2e: a1e50593     	addi	a1, a0, -0x5e2
   10a32: ef040513     	addi	a0, s0, -0x110
   10a36: 00000097     	auipc	ra, 0x0
   10a3a: 772080e7     	jalr	0x772(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E>
   10a3e: b79d         	j	0x109a4 <.Lpcrel_hi20+0xac>
   10a40: ed843583     	ld	a1, -0x128(s0)
   10a44: de043d03     	ld	s10, -0x220(s0)
   10a48: 49a1         	li	s3, 0x8
   10a4a: e1843903     	ld	s2, -0x1e8(s0)
   10a4e: c99d         	beqz	a1, 0x10a84 <.Lpcrel_hi45+0x5a>
   10a50: ed043683     	ld	a3, -0x130(s0)
   10a54: 0592         	slli	a1, a1, 0x4
   10a56: 00b68733     	add	a4, a3, a1
   10a5a: 6288         	ld	a0, 0x0(a3)
   10a5c: 03900893     	li	a7, 0x39
   10a60: 4581         	li	a1, 0x0
   10a62: 4601         	li	a2, 0x0
   10a64: 00000073     	ecall
   10a68: 6688         	ld	a0, 0x8(a3)
   10a6a: 03900893     	li	a7, 0x39
   10a6e: 4581         	li	a1, 0x0
   10a70: 4601         	li	a2, 0x0
   10a72: 00000073     	ecall
   10a76: 01068513     	addi	a0, a3, 0x10
   10a7a: 86aa         	mv	a3, a0
   10a7c: fce51fe3     	bne	a0, a4, 0x10a5a <.Lpcrel_hi45+0x30>
   10a80: f0043503     	ld	a0, -0x100(s0)
   10a84: ef043803     	ld	a6, -0x110(s0)
   10a88: ef843783     	ld	a5, -0x108(s0)
   10a8c: f0042c23     	sw	zero, -0xe8(s0)
   10a90: c929         	beqz	a0, 0x10ae2 <.Lpcrel_hi45+0xb8>
   10a92: 050e         	slli	a0, a0, 0x3
   10a94: 00a786b3     	add	a3, a5, a0
   10a98: 84be         	mv	s1, a5
   10a9a: 6098         	ld	a4, 0x0(s1)
   10a9c: f2e43823     	sd	a4, -0xd0(s0)
   10aa0: f1840593     	addi	a1, s0, -0xe8
   10aa4: 10400893     	li	a7, 0x104
   10aa8: 853a         	mv	a0, a4
   10aaa: 4601         	li	a2, 0x0
   10aac: 00000073     	ecall
   10ab0: a005         	j	0x10ad0 <.Lpcrel_hi45+0xa6>
   10ab2: 07c00893     	li	a7, 0x7c
   10ab6: 4501         	li	a0, 0x0
   10ab8: 4581         	li	a1, 0x0
   10aba: 4601         	li	a2, 0x0
   10abc: 00000073     	ecall
   10ac0: f1840593     	addi	a1, s0, -0xe8
   10ac4: 10400893     	li	a7, 0x104
   10ac8: 853a         	mv	a0, a4
   10aca: 4601         	li	a2, 0x0
   10acc: 00000073     	ecall
   10ad0: f2a43023     	sd	a0, -0xe0(s0)
   10ad4: fd750fe3     	beq	a0, s7, 0x10ab2 <.Lpcrel_hi45+0x88>
   10ad8: 46a71c63     	bne	a4, a0, 0x10f50 <.Lpcrel_hi32+0x3e>
   10adc: 04a1         	addi	s1, s1, 0x8
   10ade: fad49ee3     	bne	s1, a3, 0x10a9a <.Lpcrel_hi45+0x70>
   10ae2: 00080a63     	beqz	a6, 0x10af6 <.Lpcrel_hi45+0xcc>
   10ae6: 00381593     	slli	a1, a6, 0x3
   10aea: 4621         	li	a2, 0x8
   10aec: 853e         	mv	a0, a5
   10aee: 00001097     	auipc	ra, 0x1
   10af2: 01a080e7     	jalr	0x1a(ra) <__rust_dealloc>
   10af6: ec843583     	ld	a1, -0x138(s0)
   10afa: c989         	beqz	a1, 0x10b0c <.Lpcrel_hi45+0xe2>
   10afc: ed043503     	ld	a0, -0x130(s0)
   10b00: 0592         	slli	a1, a1, 0x4
   10b02: 4621         	li	a2, 0x8
   10b04: 00001097     	auipc	ra, 0x1
   10b08: 004080e7     	jalr	0x4(ra) <__rust_dealloc>
   10b0c: e4043423     	sd	zero, -0x1b8(s0)
   10b10: eb040513     	addi	a0, s0, -0x150
   10b14: 00001097     	auipc	ra, 0x1
   10b18: 8e4080e7     	jalr	-0x71c(ra) <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE>
   10b1c: 00090d63     	beqz	s2, 0x10b36 <.Lpcrel_hi45+0x10c>
   10b20: 00591513     	slli	a0, s2, 0x5
   10b24: 00791593     	slli	a1, s2, 0x7
   10b28: 8d89         	sub	a1, a1, a0
   10b2a: 4621         	li	a2, 0x8
   10b2c: 8552         	mv	a0, s4
   10b2e: 00001097     	auipc	ra, 0x1
   10b32: fda080e7     	jalr	-0x26(ra) <__rust_dealloc>
   10b36: e5043583     	ld	a1, -0x1b0(s0)
   10b3a: b0058be3     	beqz	a1, 0x10650 <.Lpcrel_hi19+0x1e>
   10b3e: 0592         	slli	a1, a1, 0x4
   10b40: 4621         	li	a2, 0x8
   10b42: 856a         	mv	a0, s10
   10b44: 00001097     	auipc	ra, 0x1
   10b48: fc4080e7     	jalr	-0x3c(ra) <__rust_dealloc>
   10b4c: b611         	j	0x10650 <.Lpcrel_hi19+0x1e>
   10b4e: 01f77613     	andi	a2, a4, 0x1f
   10b52: 01967b63     	bgeu	a2, s9, 0x10b68 <.Lpcrel_hi45+0x13e>
   10b56: a00d         	j	0x10b78 <.Lpcrel_hi45+0x14e>
   10b58: 00f7f693     	andi	a3, a5, 0xf
   10b5c: 069a         	slli	a3, a3, 0x6
   10b5e: 03f67613     	andi	a2, a2, 0x3f
   10b62: 8e55         	or	a2, a2, a3
   10b64: 01966a63     	bltu	a2, s9, 0x10b78 <.Lpcrel_hi45+0x14e>
   10b68: 55f9         	li	a1, -0x2
   10b6a: 02000693     	li	a3, 0x20
   10b6e: 00d66563     	bltu	a2, a3, 0x10b78 <.Lpcrel_hi45+0x14e>
   10b72: 40063593     	sltiu	a1, a2, 0x400
   10b76: 15f1         	addi	a1, a1, -0x4
   10b78: 952e         	add	a0, a0, a1
   10b7a: e4a43423     	sd	a0, -0x1b8(s0)
   10b7e: b639         	j	0x1068c <.Lpcrel_hi19+0x5a>
   10b80: e3840513     	addi	a0, s0, -0x1c8
   10b84: 4609         	li	a2, 0x2
   10b86: 4685         	li	a3, 0x1
   10b88: 4705         	li	a4, 0x1
   10b8a: 00000097     	auipc	ra, 0x0
   10b8e: 78a080e7     	jalr	0x78a(ra) <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E>
   10b92: e4843583     	ld	a1, -0x1b8(s0)
   10b96: bb75         	j	0x10952 <.Lpcrel_hi20+0x5a>
   10b98: c595         	beqz	a1, 0x10bc4 <.Lpcrel_hi45+0x19a>
   10b9a: 06050513     	addi	a0, a0, 0x60
   10b9e: 19fd         	addi	s3, s3, -0x1
   10ba0: fa058593     	addi	a1, a1, -0x60
   10ba4: d00985e3     	beqz	s3, 0x108ae <.Lpcrel_hi21+0x8a>
   10ba8: 6d14         	ld	a3, 0x18(a0)
   10baa: e299         	bnez	a3, 0x10bb0 <.Lpcrel_hi45+0x186>
   10bac: 6114         	ld	a3, 0x0(a0)
   10bae: d6ed         	beqz	a3, 0x10b98 <.Lpcrel_hi45+0x16e>
   10bb0: cdd1         	beqz	a1, 0x10c4c <.Lpcrel_hi23>
   10bb2: 4601         	li	a2, 0x0
   10bb4: 06050513     	addi	a0, a0, 0x60
   10bb8: 19fd         	addi	s3, s3, -0x1
   10bba: fa058593     	addi	a1, a1, -0x60
   10bbe: fe0995e3     	bnez	s3, 0x10ba8 <.Lpcrel_hi45+0x17e>
   10bc2: b1f5         	j	0x108ae <.Lpcrel_hi21+0x8a>
   10bc4: 8a05         	andi	a2, a2, 0x1
   10bc6: c259         	beqz	a2, 0x10c4c <.Lpcrel_hi23>
   10bc8: ec043423     	sd	zero, -0x138(s0)
   10bcc: ed943823     	sd	s9, -0x130(s0)
   10bd0: ec043c23     	sd	zero, -0x128(s0)
   10bd4: 84ba         	mv	s1, a4
   10bd6: 4c89         	li	s9, 0x2
   10bd8: dce43c23     	sd	a4, -0x228(s0)
   10bdc: e31d         	bnez	a4, 0x10c02 <.Lpcrel_hi45+0x1d8>
   10bde: b37d         	j	0x1098c <.Lpcrel_hi20+0x94>
   10be0: ed043503     	ld	a0, -0x130(s0)
   10be4: ee843583     	ld	a1, -0x118(s0)
   10be8: ee043603     	ld	a2, -0x120(s0)
   10bec: 00491693     	slli	a3, s2, 0x4
   10bf0: 9536         	add	a0, a0, a3
   10bf2: e50c         	sd	a1, 0x8(a0)
   10bf4: e110         	sd	a2, 0x0(a0)
   10bf6: 0905         	addi	s2, s2, 0x1
   10bf8: 14fd         	addi	s1, s1, -0x1
   10bfa: ed243c23     	sd	s2, -0x128(s0)
   10bfe: d80487e3     	beqz	s1, 0x1098c <.Lpcrel_hi20+0x94>
   10c02: f4043023     	sd	zero, -0xc0(s0)
   10c06: f2043c23     	sd	zero, -0xc8(s0)
   10c0a: f3840513     	addi	a0, s0, -0xc8
   10c0e: 03b00893     	li	a7, 0x3b
   10c12: 4581         	li	a1, 0x0
   10c14: 4601         	li	a2, 0x0
   10c16: 00000073     	ecall
   10c1a: f3843503     	ld	a0, -0xc8(s0)
   10c1e: f4043583     	ld	a1, -0xc0(s0)
   10c22: ed843903     	ld	s2, -0x128(s0)
   10c26: ec843603     	ld	a2, -0x138(s0)
   10c2a: eea43023     	sd	a0, -0x120(s0)
   10c2e: eeb43423     	sd	a1, -0x118(s0)
   10c32: fac917e3     	bne	s2, a2, 0x10be0 <.Lpcrel_hi45+0x1b6>

0000000000010c36 <.Lpcrel_hi46>:
   10c36: 00005517     	auipc	a0, 0x5
   10c3a: 82a50593     	addi	a1, a0, -0x7d6
   10c3e: ec840513     	addi	a0, s0, -0x138
   10c42: 00000097     	auipc	ra, 0x0
   10c46: 61c080e7     	jalr	0x61c(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E>
   10c4a: bf59         	j	0x10be0 <.Lpcrel_hi45+0x1b6>

0000000000010c4c <.Lpcrel_hi23>:
   10c4c: 00004517     	auipc	a0, 0x4
   10c50: 60450513     	addi	a0, a0, 0x604
   10c54: f2a43c23     	sd	a0, -0xc8(s0)
   10c58: f5643023     	sd	s6, -0xc0(s0)
   10c5c: f4043c23     	sd	zero, -0xa8(s0)
   10c60: 49a1         	li	s3, 0x8
   10c62: f5343423     	sd	s3, -0xb8(s0)
   10c66: f4043823     	sd	zero, -0xb0(s0)
   10c6a: f3840513     	addi	a0, s0, -0xc8
   10c6e: 00001097     	auipc	ra, 0x1
   10c72: 366080e7     	jalr	0x366(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   10c76: 4c89         	li	s9, 0x2
   10c78: e4043423     	sd	zero, -0x1b8(s0)
   10c7c: eb040513     	addi	a0, s0, -0x150
   10c80: 00000097     	auipc	ra, 0x0
   10c84: 778080e7     	jalr	0x778(ra) <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE>
   10c88: e8091ce3     	bnez	s2, 0x10b20 <.Lpcrel_hi45+0xf6>
   10c8c: b56d         	j	0x10b36 <.Lpcrel_hi45+0x10c>
   10c8e: 012a0733     	add	a4, s4, s2
   10c92: f0e43823     	sd	a4, -0xf0(s0)
   10c96: 01870513     	addi	a0, a4, 0x18
   10c9a: f0a43c23     	sd	a0, -0xe8(s0)
   10c9e: 6b08         	ld	a0, 0x10(a4)
   10ca0: e1843903     	ld	s2, -0x1e8(s0)
   10ca4: c529         	beqz	a0, 0x10cee <.Lpcrel_hi23+0xa2>
   10ca6: 670c         	ld	a1, 0x8(a4)
   10ca8: f9c00513     	li	a0, -0x64
   10cac: 03800893     	li	a7, 0x38
   10cb0: 57fd         	li	a5, -0x1
   10cb2: 4601         	li	a2, 0x0
   10cb4: 00000073     	ecall
   10cb8: 22f50463     	beq	a0, a5, 0x10ee0 <.Lpcrel_hi42+0xd4>
   10cbc: 86aa         	mv	a3, a0
   10cbe: 03900893     	li	a7, 0x39
   10cc2: 4501         	li	a0, 0x0
   10cc4: 4581         	li	a1, 0x0
   10cc6: 4601         	li	a2, 0x0
   10cc8: 00000073     	ecall
   10ccc: 48e1         	li	a7, 0x18
   10cce: 8536         	mv	a0, a3
   10cd0: 4581         	li	a1, 0x0
   10cd2: 4601         	li	a2, 0x0
   10cd4: 00000073     	ecall
   10cd8: f2a43023     	sd	a0, -0xe0(s0)
   10cdc: 2a051963     	bnez	a0, 0x10f8e <.Lpcrel_hi44+0x14>
   10ce0: 03900893     	li	a7, 0x39
   10ce4: 8536         	mv	a0, a3
   10ce6: 4581         	li	a1, 0x0
   10ce8: 4601         	li	a2, 0x0
   10cea: 00000073     	ecall
   10cee: 7708         	ld	a0, 0x28(a4)
   10cf0: c539         	beqz	a0, 0x10d3e <.Lpcrel_hi23+0xf2>
   10cf2: 730c         	ld	a1, 0x20(a4)
   10cf4: f9c00513     	li	a0, -0x64
   10cf8: 20100613     	li	a2, 0x201
   10cfc: 03800893     	li	a7, 0x38
   10d00: 57fd         	li	a5, -0x1
   10d02: 00000073     	ecall
   10d06: 1ef50c63     	beq	a0, a5, 0x10efe <.Lpcrel_hi28+0xa>
   10d0a: 86aa         	mv	a3, a0
   10d0c: 4785         	li	a5, 0x1
   10d0e: 4505         	li	a0, 0x1
   10d10: 03900893     	li	a7, 0x39
   10d14: 4581         	li	a1, 0x0
   10d16: 4601         	li	a2, 0x0
   10d18: 00000073     	ecall
   10d1c: 48e1         	li	a7, 0x18
   10d1e: 8536         	mv	a0, a3
   10d20: 4581         	li	a1, 0x0
   10d22: 4601         	li	a2, 0x0
   10d24: 00000073     	ecall
   10d28: f2a43023     	sd	a0, -0xe0(s0)
   10d2c: 2af51763     	bne	a0, a5, 0x10fda <.Lpcrel_hi37+0x1a>
   10d30: 03900893     	li	a7, 0x39
   10d34: 8536         	mv	a0, a3
   10d36: 4581         	li	a1, 0x0
   10d38: 4601         	li	a2, 0x0
   10d3a: 00000073     	ecall
   10d3e: 02098d63     	beqz	s3, 0x10d78 <.Lpcrel_hi23+0x12c>
   10d42: 03900893     	li	a7, 0x39
   10d46: 4501         	li	a0, 0x0
   10d48: 4581         	li	a1, 0x0
   10d4a: 4601         	li	a2, 0x0
   10d4c: 00000073     	ecall
   10d50: ed843503     	ld	a0, -0x128(s0)
   10d54: fff9c593     	not	a1, s3
   10d58: 2ea5fb63     	bgeu	a1, a0, 0x1104e <.Lpcrel_hi35>
   10d5c: ed043503     	ld	a0, -0x130(s0)
   10d60: 956a         	add	a0, a0, s10
   10d62: fe853503     	ld	a0, -0x18(a0)
   10d66: 48e1         	li	a7, 0x18
   10d68: 4581         	li	a1, 0x0
   10d6a: 4601         	li	a2, 0x0
   10d6c: 00000073     	ecall
   10d70: f2a43023     	sd	a0, -0xe0(s0)
   10d74: 24051063     	bnez	a0, 0x10fb4 <.Lpcrel_hi30+0x1a>
   10d78: 413007b3     	neg	a5, s3
   10d7c: dd843503     	ld	a0, -0x228(s0)
   10d80: 02a7fb63     	bgeu	a5, a0, 0x10db6 <.Lpcrel_hi23+0x16a>
   10d84: 4505         	li	a0, 0x1
   10d86: 03900893     	li	a7, 0x39
   10d8a: 4581         	li	a1, 0x0
   10d8c: 4601         	li	a2, 0x0
   10d8e: 00000073     	ecall
   10d92: ed843503     	ld	a0, -0x128(s0)
   10d96: 4685         	li	a3, 0x1
   10d98: 2ca7f363     	bgeu	a5, a0, 0x1105e <.Lpcrel_hi38>
   10d9c: ed043503     	ld	a0, -0x130(s0)
   10da0: 956a         	add	a0, a0, s10
   10da2: 6108         	ld	a0, 0x0(a0)
   10da4: 48e1         	li	a7, 0x18
   10da6: 4581         	li	a1, 0x0
   10da8: 4601         	li	a2, 0x0
   10daa: 00000073     	ecall
   10dae: f2a43023     	sd	a0, -0xe0(s0)
   10db2: 24d51763     	bne	a0, a3, 0x11000 <.Lpcrel_hi34+0x1a>
   10db6: ed843503     	ld	a0, -0x128(s0)
   10dba: c90d         	beqz	a0, 0x10dec <.Lpcrel_hi23+0x1a0>
   10dbc: ed043683     	ld	a3, -0x130(s0)
   10dc0: 0512         	slli	a0, a0, 0x4
   10dc2: 00a687b3     	add	a5, a3, a0
   10dc6: 6288         	ld	a0, 0x0(a3)
   10dc8: 03900893     	li	a7, 0x39
   10dcc: 4581         	li	a1, 0x0
   10dce: 4601         	li	a2, 0x0
   10dd0: 00000073     	ecall
   10dd4: 6688         	ld	a0, 0x8(a3)
   10dd6: 03900893     	li	a7, 0x39
   10dda: 4581         	li	a1, 0x0
   10ddc: 4601         	li	a2, 0x0
   10dde: 00000073     	ecall
   10de2: 01068513     	addi	a0, a3, 0x10
   10de6: 86aa         	mv	a3, a0
   10de8: fcf51fe3     	bne	a0, a5, 0x10dc6 <.Lpcrel_hi23+0x17a>
   10dec: 6328         	ld	a0, 0x40(a4)
   10dee: de043483     	ld	s1, -0x220(s0)
   10df2: 24050463     	beqz	a0, 0x1103a <.Lpcrel_hi41>
   10df6: 7f08         	ld	a0, 0x38(a4)
   10df8: 6508         	ld	a0, 0x8(a0)
   10dfa: 6b2c         	ld	a1, 0x50(a4)
   10dfc: 0dd00893     	li	a7, 0xdd
   10e00: 56fd         	li	a3, -0x1
   10e02: 4601         	li	a2, 0x0
   10e04: 00000073     	ecall
   10e08: 16d51563     	bne	a0, a3, 0x10f72 <.Lpcrel_hi43>

0000000000010e0c <.Lpcrel_hi42>:
   10e0c: 00004517     	auipc	a0, 0x4
   10e10: 4cc50513     	addi	a0, a0, 0x4cc
   10e14: f2a43c23     	sd	a0, -0xc8(s0)
   10e18: 4505         	li	a0, 0x1
   10e1a: f4a43023     	sd	a0, -0xc0(s0)
   10e1e: f4043c23     	sd	zero, -0xa8(s0)
   10e22: 4521         	li	a0, 0x8
   10e24: f4a43423     	sd	a0, -0xb8(s0)
   10e28: f4043823     	sd	zero, -0xb0(s0)
   10e2c: f3840513     	addi	a0, s0, -0xc8
   10e30: 00001097     	auipc	ra, 0x1
   10e34: 1a4080e7     	jalr	0x1a4(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   10e38: ef043583     	ld	a1, -0x110(s0)
   10e3c: c989         	beqz	a1, 0x10e4e <.Lpcrel_hi42+0x42>
   10e3e: ef843503     	ld	a0, -0x108(s0)
   10e42: 058e         	slli	a1, a1, 0x3
   10e44: 4621         	li	a2, 0x8
   10e46: 00001097     	auipc	ra, 0x1
   10e4a: cc2080e7     	jalr	-0x33e(ra) <__rust_dealloc>
   10e4e: ec843583     	ld	a1, -0x138(s0)
   10e52: c989         	beqz	a1, 0x10e64 <.Lpcrel_hi42+0x58>
   10e54: ed043503     	ld	a0, -0x130(s0)
   10e58: 0592         	slli	a1, a1, 0x4
   10e5a: 4621         	li	a2, 0x8
   10e5c: 00001097     	auipc	ra, 0x1
   10e60: cac080e7     	jalr	-0x354(ra) <__rust_dealloc>
   10e64: eb040513     	addi	a0, s0, -0x150
   10e68: 00000097     	auipc	ra, 0x0
   10e6c: 590080e7     	jalr	0x590(ra) <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE>
   10e70: 00090d63     	beqz	s2, 0x10e8a <.Lpcrel_hi42+0x7e>
   10e74: 00591513     	slli	a0, s2, 0x5
   10e78: 00791593     	slli	a1, s2, 0x7
   10e7c: 8d89         	sub	a1, a1, a0
   10e7e: 4621         	li	a2, 0x8
   10e80: 8552         	mv	a0, s4
   10e82: 00001097     	auipc	ra, 0x1
   10e86: c86080e7     	jalr	-0x37a(ra) <__rust_dealloc>
   10e8a: e5043583     	ld	a1, -0x1b0(s0)
   10e8e: c981         	beqz	a1, 0x10e9e <.Lpcrel_hi42+0x92>
   10e90: 0592         	slli	a1, a1, 0x4
   10e92: 4621         	li	a2, 0x8
   10e94: 8526         	mv	a0, s1
   10e96: 00001097     	auipc	ra, 0x1
   10e9a: c72080e7     	jalr	-0x38e(ra) <__rust_dealloc>
   10e9e: e3843583     	ld	a1, -0x1c8(s0)
   10ea2: c981         	beqz	a1, 0x10eb2 <.Lpcrel_hi42+0xa6>
   10ea4: e4043503     	ld	a0, -0x1c0(s0)
   10ea8: 4605         	li	a2, 0x1
   10eaa: 00001097     	auipc	ra, 0x1
   10eae: c5e080e7     	jalr	-0x3a2(ra) <__rust_dealloc>
   10eb2: 5571         	li	a0, -0x4
   10eb4: 22813083     	ld	ra, 0x228(sp)
   10eb8: 22013403     	ld	s0, 0x220(sp)
   10ebc: 21813483     	ld	s1, 0x218(sp)
   10ec0: 21013903     	ld	s2, 0x210(sp)
   10ec4: 20813983     	ld	s3, 0x208(sp)
   10ec8: 20013a03     	ld	s4, 0x200(sp)
   10ecc: 7afe         	ld	s5, 0x1f8(sp)
   10ece: 7b5e         	ld	s6, 0x1f0(sp)
   10ed0: 7bbe         	ld	s7, 0x1e8(sp)
   10ed2: 7c1e         	ld	s8, 0x1e0(sp)
   10ed4: 6cfe         	ld	s9, 0x1d8(sp)
   10ed6: 6d5e         	ld	s10, 0x1d0(sp)
   10ed8: 6dbe         	ld	s11, 0x1c8(sp)
   10eda: 23010113     	addi	sp, sp, 0x230
   10ede: 8082         	ret
   10ee0: f1040513     	addi	a0, s0, -0xf0
   10ee4: f2a43023     	sd	a0, -0xe0(s0)

0000000000010ee8 <.Lpcrel_hi27>:
   10ee8: 00000517     	auipc	a0, 0x0
   10eec: 1ec50513     	addi	a0, a0, 0x1ec
   10ef0: f2a43423     	sd	a0, -0xd8(s0)

0000000000010ef4 <.Lpcrel_hi28>:
   10ef4: 00004517     	auipc	a0, 0x4
   10ef8: 3ac50513     	addi	a0, a0, 0x3ac
   10efc: a839         	j	0x10f1a <.Lpcrel_hi32+0x8>
   10efe: f1840513     	addi	a0, s0, -0xe8
   10f02: f2a43023     	sd	a0, -0xe0(s0)

0000000000010f06 <.Lpcrel_hi31>:
   10f06: 00000517     	auipc	a0, 0x0
   10f0a: 1ce50513     	addi	a0, a0, 0x1ce
   10f0e: f2a43423     	sd	a0, -0xd8(s0)

0000000000010f12 <.Lpcrel_hi32>:
   10f12: 00004517     	auipc	a0, 0x4
   10f16: 38e50513     	addi	a0, a0, 0x38e
   10f1a: f2a43c23     	sd	a0, -0xc8(s0)
   10f1e: 4509         	li	a0, 0x2
   10f20: f4a43023     	sd	a0, -0xc0(s0)
   10f24: f4043c23     	sd	zero, -0xa8(s0)
   10f28: f2040513     	addi	a0, s0, -0xe0
   10f2c: f4a43423     	sd	a0, -0xb8(s0)
   10f30: 4505         	li	a0, 0x1
   10f32: f4a43823     	sd	a0, -0xb0(s0)
   10f36: f3840513     	addi	a0, s0, -0xc8
   10f3a: 00001097     	auipc	ra, 0x1
   10f3e: 09a080e7     	jalr	0x9a(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   10f42: de043483     	ld	s1, -0x220(s0)
   10f46: ef043583     	ld	a1, -0x110(s0)
   10f4a: ee059ae3     	bnez	a1, 0x10e3e <.Lpcrel_hi42+0x32>
   10f4e: b701         	j	0x10e4e <.Lpcrel_hi42+0x42>
   10f50: f2043c23     	sd	zero, -0xc8(s0)

0000000000010f54 <.Lpcrel_hi26>:
   10f54: 00004517     	auipc	a0, 0x4
   10f58: 3c450713     	addi	a4, a0, 0x3c4
   10f5c: f3040593     	addi	a1, s0, -0xd0
   10f60: f2040613     	addi	a2, s0, -0xe0
   10f64: f3840693     	addi	a3, s0, -0xc8
   10f68: 4501         	li	a0, 0x0
   10f6a: 00000097     	auipc	ra, 0x0
   10f6e: 18c080e7     	jalr	0x18c(ra) <_ZN4core9panicking13assert_failed17h3cf3161050e83bdaE>

0000000000010f72 <.Lpcrel_hi43>:
   10f72: 00004517     	auipc	a0, 0x4
   10f76: 49650513     	addi	a0, a0, 0x496

0000000000010f7a <.Lpcrel_hi44>:
   10f7a: 00004597     	auipc	a1, 0x4
   10f7e: 4b658613     	addi	a2, a1, 0x4b6
   10f82: 02800593     	li	a1, 0x28
   10f86: 00002097     	auipc	ra, 0x2
   10f8a: 9d4080e7     	jalr	-0x62c(ra) <_ZN4core9panicking5panic17h6952156bbcf3c8fdE>
   10f8e: f2043c23     	sd	zero, -0xc8(s0)

0000000000010f92 <.Lpcrel_hi29>:
   10f92: 00004517     	auipc	a0, 0x4
   10f96: 3ae50613     	addi	a2, a0, 0x3ae

0000000000010f9a <.Lpcrel_hi30>:
   10f9a: 00004517     	auipc	a0, 0x4
   10f9e: 3c650713     	addi	a4, a0, 0x3c6
   10fa2: f2040593     	addi	a1, s0, -0xe0
   10fa6: f3840693     	addi	a3, s0, -0xc8
   10faa: 4501         	li	a0, 0x0
   10fac: 00000097     	auipc	ra, 0x0
   10fb0: 14a080e7     	jalr	0x14a(ra) <_ZN4core9panicking13assert_failed17h3cf3161050e83bdaE>
   10fb4: f2043c23     	sd	zero, -0xc8(s0)

0000000000010fb8 <.Lpcrel_hi36>:
   10fb8: 00004517     	auipc	a0, 0x4
   10fbc: 38850613     	addi	a2, a0, 0x388

0000000000010fc0 <.Lpcrel_hi37>:
   10fc0: 00004517     	auipc	a0, 0x4
   10fc4: 3e850713     	addi	a4, a0, 0x3e8
   10fc8: f2040593     	addi	a1, s0, -0xe0
   10fcc: f3840693     	addi	a3, s0, -0xc8
   10fd0: 4501         	li	a0, 0x0
   10fd2: 00000097     	auipc	ra, 0x0
   10fd6: 124080e7     	jalr	0x124(ra) <_ZN4core9panicking13assert_failed17h3cf3161050e83bdaE>
   10fda: f2043c23     	sd	zero, -0xc8(s0)

0000000000010fde <.Lpcrel_hi33>:
   10fde: 00004517     	auipc	a0, 0x4
   10fe2: 35a50613     	addi	a2, a0, 0x35a

0000000000010fe6 <.Lpcrel_hi34>:
   10fe6: 00004517     	auipc	a0, 0x4
   10fea: 39250713     	addi	a4, a0, 0x392
   10fee: f2040593     	addi	a1, s0, -0xe0
   10ff2: f3840693     	addi	a3, s0, -0xc8
   10ff6: 4501         	li	a0, 0x0
   10ff8: 00000097     	auipc	ra, 0x0
   10ffc: 0fe080e7     	jalr	0xfe(ra) <_ZN4core9panicking13assert_failed17h3cf3161050e83bdaE>
   11000: f2043c23     	sd	zero, -0xc8(s0)

0000000000011004 <.Lpcrel_hi39>:
   11004: 00004517     	auipc	a0, 0x4
   11008: 33450613     	addi	a2, a0, 0x334

000000000001100c <.Lpcrel_hi40>:
   1100c: 00004517     	auipc	a0, 0x4
   11010: 3cc50713     	addi	a4, a0, 0x3cc
   11014: f2040593     	addi	a1, s0, -0xe0
   11018: f3840693     	addi	a3, s0, -0xc8
   1101c: 4501         	li	a0, 0x0
   1101e: 00000097     	auipc	ra, 0x0
   11022: 0d8080e7     	jalr	0xd8(ra) <_ZN4core9panicking13assert_failed17h3cf3161050e83bdaE>

0000000000011026 <.Lpcrel_hi22>:
   11026: 00004517     	auipc	a0, 0x4
   1102a: 0fa50613     	addi	a2, a0, 0xfa
   1102e: 8526         	mv	a0, s1
   11030: 85d6         	mv	a1, s5
   11032: 00001097     	auipc	ra, 0x1
   11036: 74a080e7     	jalr	0x74a(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

000000000001103a <.Lpcrel_hi41>:
   1103a: 00004517     	auipc	a0, 0x4
   1103e: 3b650613     	addi	a2, a0, 0x3b6
   11042: 4501         	li	a0, 0x0
   11044: 4581         	li	a1, 0x0
   11046: 00002097     	auipc	ra, 0x2
   1104a: 94e080e7     	jalr	-0x6b2(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

000000000001104e <.Lpcrel_hi35>:
   1104e: 00004517     	auipc	a0, 0x4
   11052: 34250513     	addi	a0, a0, 0x342
   11056: 00002097     	auipc	ra, 0x2
   1105a: 870080e7     	jalr	-0x790(ra) <_ZN4core6option13unwrap_failed17h78203addebfbdbebE>

000000000001105e <.Lpcrel_hi38>:
   1105e: 00004517     	auipc	a0, 0x4
   11062: 36250513     	addi	a0, a0, 0x362
   11066: 00002097     	auipc	ra, 0x2
   1106a: 860080e7     	jalr	-0x7a0(ra) <_ZN4core6option13unwrap_failed17h78203addebfbdbebE>

000000000001106e <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hcade7ebe5af7c4e1E>:
   1106e: 1141         	addi	sp, sp, -0x10
   11070: e406         	sd	ra, 0x8(sp)
   11072: e022         	sd	s0, 0x0(sp)
   11074: 0800         	addi	s0, sp, 0x10
   11076: 0245e603     	lwu	a2, 0x24(a1)
   1107a: 6108         	ld	a0, 0x0(a0)
   1107c: 01067693     	andi	a3, a2, 0x10
   11080: ea99         	bnez	a3, 0x11096 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hcade7ebe5af7c4e1E+0x28>
   11082: 02067613     	andi	a2, a2, 0x20
   11086: ee19         	bnez	a2, 0x110a4 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hcade7ebe5af7c4e1E+0x36>
   11088: 60a2         	ld	ra, 0x8(sp)
   1108a: 6402         	ld	s0, 0x0(sp)
   1108c: 0141         	addi	sp, sp, 0x10
   1108e: 00003317     	auipc	t1, 0x3
   11092: 20830067     	jr	0x208(t1) <_ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$isize$GT$3fmt17h3f90d00ed58c1f73E>
   11096: 60a2         	ld	ra, 0x8(sp)
   11098: 6402         	ld	s0, 0x0(sp)
   1109a: 0141         	addi	sp, sp, 0x10
   1109c: 00003317     	auipc	t1, 0x3
   110a0: e4630067     	jr	-0x1ba(t1) <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E>
   110a4: 60a2         	ld	ra, 0x8(sp)
   110a6: 6402         	ld	s0, 0x0(sp)
   110a8: 0141         	addi	sp, sp, 0x10
   110aa: 00003317     	auipc	t1, 0x3
   110ae: e9230067     	jr	-0x16e(t1) <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE>

00000000000110b2 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h6d022764c582d9b2E>:
   110b2: 1141         	addi	sp, sp, -0x10
   110b4: e406         	sd	ra, 0x8(sp)
   110b6: e022         	sd	s0, 0x0(sp)
   110b8: 0800         	addi	s0, sp, 0x10
   110ba: 6110         	ld	a2, 0x0(a0)
   110bc: 6514         	ld	a3, 0x8(a0)
   110be: 872e         	mv	a4, a1
   110c0: 8532         	mv	a0, a2
   110c2: 85b6         	mv	a1, a3
   110c4: 863a         	mv	a2, a4
   110c6: 60a2         	ld	ra, 0x8(sp)
   110c8: 6402         	ld	s0, 0x0(sp)
   110ca: 0141         	addi	sp, sp, 0x10
   110cc: 00002317     	auipc	t1, 0x2
   110d0: 6fe30067     	jr	0x6fe(t1) <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h6c34711bbfb2649bE>

00000000000110d4 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17hb7d71b3ad02f0fcdE>:
   110d4: 1141         	addi	sp, sp, -0x10
   110d6: e406         	sd	ra, 0x8(sp)
   110d8: e022         	sd	s0, 0x0(sp)
   110da: 0800         	addi	s0, sp, 0x10
   110dc: 6110         	ld	a2, 0x0(a0)
   110de: 6608         	ld	a0, 0x8(a2)
   110e0: 6a10         	ld	a2, 0x10(a2)
   110e2: 86ae         	mv	a3, a1
   110e4: 85b2         	mv	a1, a2
   110e6: 8636         	mv	a2, a3
   110e8: 60a2         	ld	ra, 0x8(sp)
   110ea: 6402         	ld	s0, 0x0(sp)
   110ec: 0141         	addi	sp, sp, 0x10
   110ee: 00002317     	auipc	t1, 0x2
   110f2: 6dc30067     	jr	0x6dc(t1) <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h6c34711bbfb2649bE>

00000000000110f6 <_ZN4core9panicking13assert_failed17h3cf3161050e83bdaE>:
   110f6: 1101         	addi	sp, sp, -0x20
   110f8: ec06         	sd	ra, 0x18(sp)
   110fa: e822         	sd	s0, 0x10(sp)
   110fc: 1000         	addi	s0, sp, 0x20
   110fe: 883a         	mv	a6, a4
   11100: 87b6         	mv	a5, a3
   11102: feb43023     	sd	a1, -0x20(s0)
   11106: fec43423     	sd	a2, -0x18(s0)

000000000001110a <.Lpcrel_hi1>:
   1110a: 00004597     	auipc	a1, 0x4
   1110e: 38658613     	addi	a2, a1, 0x386
   11112: fe040593     	addi	a1, s0, -0x20
   11116: fe840693     	addi	a3, s0, -0x18
   1111a: 8732         	mv	a4, a2
   1111c: 00002097     	auipc	ra, 0x2
   11120: 8c8080e7     	jalr	-0x738(ra) <_ZN4core9panicking19assert_failed_inner17hc4a20c2ec30e9d4dE>

0000000000011124 <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708>:
   11124: 7179         	addi	sp, sp, -0x30
   11126: f406         	sd	ra, 0x28(sp)
   11128: f022         	sd	s0, 0x20(sp)
   1112a: ec26         	sd	s1, 0x18(sp)
   1112c: e84a         	sd	s2, 0x10(sp)
   1112e: e44e         	sd	s3, 0x8(sp)
   11130: 1800         	addi	s0, sp, 0x30
   11132: 6698         	ld	a4, 0x8(a3)
   11134: 8932         	mv	s2, a2
   11136: 89ae         	mv	s3, a1
   11138: 84aa         	mv	s1, a0
   1113a: cb15         	beqz	a4, 0x1116e <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708+0x4a>
   1113c: 6a8c         	ld	a1, 0x10(a3)
   1113e: cd9d         	beqz	a1, 0x1117c <.Lpcrel_hi3+0xa>
   11140: 6288         	ld	a0, 0x0(a3)
   11142: 864e         	mv	a2, s3
   11144: 86ca         	mv	a3, s2
   11146: 00001097     	auipc	ra, 0x1
   1114a: 9e8080e7     	jalr	-0x618(ra) <__rust_realloc>
   1114e: 00153593     	seqz	a1, a0
   11152: c111         	beqz	a0, 0x11156 <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708+0x32>
   11154: 89aa         	mv	s3, a0
   11156: 0134b423     	sd	s3, 0x8(s1)
   1115a: 0124b823     	sd	s2, 0x10(s1)
   1115e: e08c         	sd	a1, 0x0(s1)
   11160: 70a2         	ld	ra, 0x28(sp)
   11162: 7402         	ld	s0, 0x20(sp)
   11164: 64e2         	ld	s1, 0x18(sp)
   11166: 6942         	ld	s2, 0x10(sp)
   11168: 69a2         	ld	s3, 0x8(sp)
   1116a: 6145         	addi	sp, sp, 0x30
   1116c: 8082         	ret
   1116e: 02090763     	beqz	s2, 0x1119c <.Lpcrel_hi2+0x1c>

0000000000011172 <.Lpcrel_hi3>:
   11172: 0000e517     	auipc	a0, 0xe
   11176: fb754003     	lbu	zero, -0x49(a0)
   1117a: a039         	j	0x11188 <.Lpcrel_hi2+0x8>
   1117c: 02090063     	beqz	s2, 0x1119c <.Lpcrel_hi2+0x1c>

0000000000011180 <.Lpcrel_hi2>:
   11180: 0000e517     	auipc	a0, 0xe
   11184: fa954003     	lbu	zero, -0x57(a0)
   11188: 854a         	mv	a0, s2
   1118a: 85ce         	mv	a1, s3
   1118c: 00001097     	auipc	ra, 0x1
   11190: 958080e7     	jalr	-0x6a8(ra) <__rust_alloc>
   11194: 00153593     	seqz	a1, a0
   11198: fd55         	bnez	a0, 0x11154 <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708+0x30>
   1119a: bf75         	j	0x11156 <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708+0x32>
   1119c: 854e         	mv	a0, s3
   1119e: 0019b593     	seqz	a1, s3
   111a2: fa098ae3     	beqz	s3, 0x11156 <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708+0x32>
   111a6: b77d         	j	0x11154 <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708+0x30>

00000000000111a8 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E>:
   111a8: 711d         	addi	sp, sp, -0x60
   111aa: ec86         	sd	ra, 0x58(sp)
   111ac: e8a2         	sd	s0, 0x50(sp)
   111ae: e4a6         	sd	s1, 0x48(sp)
   111b0: e0ca         	sd	s2, 0x40(sp)
   111b2: fc4e         	sd	s3, 0x38(sp)
   111b4: 1080         	addi	s0, sp, 0x60
   111b6: 84aa         	mv	s1, a0
   111b8: 6114         	ld	a3, 0x0(a0)
   111ba: 00168613     	addi	a2, a3, 0x1
   111be: 00169513     	slli	a0, a3, 0x1
   111c2: 892e         	mv	s2, a1
   111c4: 02a67463     	bgeu	a2, a0, 0x111ec <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x44>
   111c8: 4591         	li	a1, 0x4
   111ca: 89aa         	mv	s3, a0
   111cc: 02a5f563     	bgeu	a1, a0, 0x111f6 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x4e>
   111d0: 03d55593     	srli	a1, a0, 0x3d
   111d4: 4501         	li	a0, 0x0
   111d6: e58d         	bnez	a1, 0x11200 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x58>
   111d8: 00399613     	slli	a2, s3, 0x3
   111dc: 55c5         	li	a1, -0xf
   111de: 0015d713     	srli	a4, a1, 0x1
   111e2: 06c76963     	bltu	a4, a2, 0x11254 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0xac>
   111e6: e295         	bnez	a3, 0x1120a <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x62>
   111e8: 4501         	li	a0, 0x0
   111ea: a03d         	j	0x11218 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x70>
   111ec: 8532         	mv	a0, a2
   111ee: 4591         	li	a1, 0x4
   111f0: 89aa         	mv	s3, a0
   111f2: fcc5efe3     	bltu	a1, a2, 0x111d0 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x28>
   111f6: 4991         	li	s3, 0x4
   111f8: 03d55593     	srli	a1, a0, 0x3d
   111fc: 4501         	li	a0, 0x0
   111fe: dde9         	beqz	a1, 0x111d8 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0x30>
   11200: 864a         	mv	a2, s2
   11202: 00001097     	auipc	ra, 0x1
   11206: 57a080e7     	jalr	0x57a(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>
   1120a: 6488         	ld	a0, 0x8(s1)
   1120c: 068e         	slli	a3, a3, 0x3
   1120e: fca43023     	sd	a0, -0x40(s0)
   11212: fcd43823     	sd	a3, -0x30(s0)
   11216: 4521         	li	a0, 0x8
   11218: fca43423     	sd	a0, -0x38(s0)
   1121c: fa840513     	addi	a0, s0, -0x58
   11220: 45a1         	li	a1, 0x8
   11222: fc040693     	addi	a3, s0, -0x40
   11226: 00000097     	auipc	ra, 0x0
   1122a: efe080e7     	jalr	-0x102(ra) <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708>
   1122e: fa843503     	ld	a0, -0x58(s0)
   11232: ed09         	bnez	a0, 0x1124c <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hac652fe886d56418E+0xa4>
   11234: fb043503     	ld	a0, -0x50(s0)
   11238: e488         	sd	a0, 0x8(s1)
   1123a: 0134b023     	sd	s3, 0x0(s1)
   1123e: 60e6         	ld	ra, 0x58(sp)
   11240: 6446         	ld	s0, 0x50(sp)
   11242: 64a6         	ld	s1, 0x48(sp)
   11244: 6906         	ld	s2, 0x40(sp)
   11246: 79e2         	ld	s3, 0x38(sp)
   11248: 6125         	addi	sp, sp, 0x60
   1124a: 8082         	ret
   1124c: fb043503     	ld	a0, -0x50(s0)
   11250: fb843583     	ld	a1, -0x48(s0)
   11254: 864a         	mv	a2, s2
   11256: 00001097     	auipc	ra, 0x1
   1125a: 526080e7     	jalr	0x526(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

000000000001125e <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E>:
   1125e: 711d         	addi	sp, sp, -0x60
   11260: ec86         	sd	ra, 0x58(sp)
   11262: e8a2         	sd	s0, 0x50(sp)
   11264: e4a6         	sd	s1, 0x48(sp)
   11266: e0ca         	sd	s2, 0x40(sp)
   11268: fc4e         	sd	s3, 0x38(sp)
   1126a: 1080         	addi	s0, sp, 0x60
   1126c: 84aa         	mv	s1, a0
   1126e: 6114         	ld	a3, 0x0(a0)
   11270: 00168613     	addi	a2, a3, 0x1
   11274: 00169513     	slli	a0, a3, 0x1
   11278: 892e         	mv	s2, a1
   1127a: 02a67463     	bgeu	a2, a0, 0x112a2 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x44>
   1127e: 4591         	li	a1, 0x4
   11280: 89aa         	mv	s3, a0
   11282: 02a5f563     	bgeu	a1, a0, 0x112ac <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x4e>
   11286: 03c55593     	srli	a1, a0, 0x3c
   1128a: 4501         	li	a0, 0x0
   1128c: e58d         	bnez	a1, 0x112b6 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x58>
   1128e: 00499613     	slli	a2, s3, 0x4
   11292: 55c5         	li	a1, -0xf
   11294: 0015d713     	srli	a4, a1, 0x1
   11298: 06c76963     	bltu	a4, a2, 0x1130a <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0xac>
   1129c: e295         	bnez	a3, 0x112c0 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x62>
   1129e: 4501         	li	a0, 0x0
   112a0: a03d         	j	0x112ce <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x70>
   112a2: 8532         	mv	a0, a2
   112a4: 4591         	li	a1, 0x4
   112a6: 89aa         	mv	s3, a0
   112a8: fcc5efe3     	bltu	a1, a2, 0x11286 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x28>
   112ac: 4991         	li	s3, 0x4
   112ae: 03c55593     	srli	a1, a0, 0x3c
   112b2: 4501         	li	a0, 0x0
   112b4: dde9         	beqz	a1, 0x1128e <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0x30>
   112b6: 864a         	mv	a2, s2
   112b8: 00001097     	auipc	ra, 0x1
   112bc: 4c4080e7     	jalr	0x4c4(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>
   112c0: 6488         	ld	a0, 0x8(s1)
   112c2: 0692         	slli	a3, a3, 0x4
   112c4: fca43023     	sd	a0, -0x40(s0)
   112c8: fcd43823     	sd	a3, -0x30(s0)
   112cc: 4521         	li	a0, 0x8
   112ce: fca43423     	sd	a0, -0x38(s0)
   112d2: fa840513     	addi	a0, s0, -0x58
   112d6: 45a1         	li	a1, 0x8
   112d8: fc040693     	addi	a3, s0, -0x40
   112dc: 00000097     	auipc	ra, 0x0
   112e0: e48080e7     	jalr	-0x1b8(ra) <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708>
   112e4: fa843503     	ld	a0, -0x58(s0)
   112e8: ed09         	bnez	a0, 0x11302 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hc116ee9483469eb4E+0xa4>
   112ea: fb043503     	ld	a0, -0x50(s0)
   112ee: e488         	sd	a0, 0x8(s1)
   112f0: 0134b023     	sd	s3, 0x0(s1)
   112f4: 60e6         	ld	ra, 0x58(sp)
   112f6: 6446         	ld	s0, 0x50(sp)
   112f8: 64a6         	ld	s1, 0x48(sp)
   112fa: 6906         	ld	s2, 0x40(sp)
   112fc: 79e2         	ld	s3, 0x38(sp)
   112fe: 6125         	addi	sp, sp, 0x60
   11300: 8082         	ret
   11302: fb043503     	ld	a0, -0x50(s0)
   11306: fb843583     	ld	a1, -0x48(s0)
   1130a: 864a         	mv	a2, s2
   1130c: 00001097     	auipc	ra, 0x1
   11310: 470080e7     	jalr	0x470(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

0000000000011314 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E>:
   11314: 715d         	addi	sp, sp, -0x50
   11316: e486         	sd	ra, 0x48(sp)
   11318: e0a2         	sd	s0, 0x40(sp)
   1131a: fc26         	sd	s1, 0x38(sp)
   1131c: f84a         	sd	s2, 0x30(sp)
   1131e: 0880         	addi	s0, sp, 0x50
   11320: cf55         	beqz	a4, 0x113dc <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0xc8>
   11322: 962e         	add	a2, a2, a1
   11324: 0ab66c63     	bltu	a2, a1, 0x113dc <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0xc8>
   11328: 87b6         	mv	a5, a3
   1132a: 892a         	mv	s2, a0
   1132c: 00053803     	ld	a6, 0x0(a0)
   11330: 00181513     	slli	a0, a6, 0x1
   11334: 00a66f63     	bltu	a2, a0, 0x11352 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x3e>
   11338: 8532         	mv	a0, a2
   1133a: 4585         	li	a1, 0x1
   1133c: 40100613     	li	a2, 0x401
   11340: 4491         	li	s1, 0x4
   11342: 00c77e63     	bgeu	a4, a2, 0x1135e <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x4a>
   11346: 00b71f63     	bne	a4, a1, 0x11364 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x50>
   1134a: 44a1         	li	s1, 0x8
   1134c: 00957e63     	bgeu	a0, s1, 0x11368 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x54>
   11350: a829         	j	0x1136a <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x56>
   11352: 4585         	li	a1, 0x1
   11354: 40100613     	li	a2, 0x401
   11358: 4491         	li	s1, 0x4
   1135a: fec766e3     	bltu	a4, a2, 0x11346 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x32>
   1135e: 4485         	li	s1, 0x1
   11360: feb705e3     	beq	a4, a1, 0x1134a <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x36>
   11364: 00956363     	bltu	a0, s1, 0x1136a <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x56>
   11368: 84aa         	mv	s1, a0
   1136a: 00e78533     	add	a0, a5, a4
   1136e: 157d         	addi	a0, a0, -0x1
   11370: 40f005b3     	neg	a1, a5
   11374: 8de9         	and	a1, a1, a0
   11376: 0295b633     	mulhu	a2, a1, s1
   1137a: 4501         	li	a0, 0x0
   1137c: e22d         	bnez	a2, 0x113de <.Lpcrel_hi5>
   1137e: 02958633     	mul	a2, a1, s1
   11382: 55fd         	li	a1, -0x1
   11384: 15fe         	slli	a1, a1, 0x3f
   11386: 40f586b3     	sub	a3, a1, a5
   1138a: 04c6ea63     	bltu	a3, a2, 0x113de <.Lpcrel_hi5>
   1138e: 00081463     	bnez	a6, 0x11396 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x82>
   11392: 4501         	li	a0, 0x0
   11394: a811         	j	0x113a8 <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E+0x94>
   11396: 00893503     	ld	a0, 0x8(s2)
   1139a: 02e805b3     	mul	a1, a6, a4
   1139e: fca43423     	sd	a0, -0x38(s0)
   113a2: fcb43c23     	sd	a1, -0x28(s0)
   113a6: 853e         	mv	a0, a5
   113a8: fca43823     	sd	a0, -0x30(s0)
   113ac: fb040513     	addi	a0, s0, -0x50
   113b0: fc840693     	addi	a3, s0, -0x38
   113b4: 85be         	mv	a1, a5
   113b6: 00000097     	auipc	ra, 0x0
   113ba: d6e080e7     	jalr	-0x292(ra) <_ZN5alloc7raw_vec11finish_grow17he3bf5780e6802affE.llvm.17998766461498348708>
   113be: fb043503     	ld	a0, -0x50(s0)
   113c2: e515         	bnez	a0, 0x113ee <.Lpcrel_hi5+0x10>
   113c4: fb843503     	ld	a0, -0x48(s0)
   113c8: 00a93423     	sd	a0, 0x8(s2)
   113cc: 00993023     	sd	s1, 0x0(s2)
   113d0: 60a6         	ld	ra, 0x48(sp)
   113d2: 6406         	ld	s0, 0x40(sp)
   113d4: 74e2         	ld	s1, 0x38(sp)
   113d6: 7942         	ld	s2, 0x30(sp)
   113d8: 6161         	addi	sp, sp, 0x50
   113da: 8082         	ret
   113dc: 4501         	li	a0, 0x0

00000000000113de <.Lpcrel_hi5>:
   113de: 00004617     	auipc	a2, 0x4
   113e2: 15260613     	addi	a2, a2, 0x152
   113e6: 00001097     	auipc	ra, 0x1
   113ea: 396080e7     	jalr	0x396(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>
   113ee: fb843503     	ld	a0, -0x48(s0)
   113f2: fc043583     	ld	a1, -0x40(s0)
   113f6: b7e5         	j	0x113de <.Lpcrel_hi5>

00000000000113f8 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE>:
   113f8: 7139         	addi	sp, sp, -0x40
   113fa: fc06         	sd	ra, 0x38(sp)
   113fc: f822         	sd	s0, 0x30(sp)
   113fe: f426         	sd	s1, 0x28(sp)
   11400: f04a         	sd	s2, 0x20(sp)
   11402: ec4e         	sd	s3, 0x18(sp)
   11404: e852         	sd	s4, 0x10(sp)
   11406: e456         	sd	s5, 0x8(sp)
   11408: e05a         	sd	s6, 0x0(sp)
   1140a: 0080         	addi	s0, sp, 0x40
   1140c: 01053983     	ld	s3, 0x10(a0)
   11410: 0a098463     	beqz	s3, 0x114b8 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0xc0>
   11414: 00853903     	ld	s2, 0x8(a0)
   11418: 4a01         	li	s4, 0x0
   1141a: a021         	j	0x11422 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x2a>
   1141c: 0a05         	addi	s4, s4, 0x1
   1141e: 093a0d63     	beq	s4, s3, 0x114b8 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0xc0>
   11422: 005a1513     	slli	a0, s4, 0x5
   11426: 007a1593     	slli	a1, s4, 0x7
   1142a: 8d89         	sub	a1, a1, a0
   1142c: 00b90ab3     	add	s5, s2, a1
   11430: 000ab583     	ld	a1, 0x0(s5)
   11434: c981         	beqz	a1, 0x11444 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x4c>
   11436: 008ab503     	ld	a0, 0x8(s5)
   1143a: 4605         	li	a2, 0x1
   1143c: 00000097     	auipc	ra, 0x0
   11440: 6cc080e7     	jalr	0x6cc(ra) <__rust_dealloc>
   11444: 018ab583     	ld	a1, 0x18(s5)
   11448: c981         	beqz	a1, 0x11458 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x60>
   1144a: 020ab503     	ld	a0, 0x20(s5)
   1144e: 4605         	li	a2, 0x1
   11450: 00000097     	auipc	ra, 0x0
   11454: 6b8080e7     	jalr	0x6b8(ra) <__rust_dealloc>
   11458: 040abb03     	ld	s6, 0x40(s5)
   1145c: 020b0463     	beqz	s6, 0x11484 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x8c>
   11460: 038ab483     	ld	s1, 0x38(s5)
   11464: 04a1         	addi	s1, s1, 0x8
   11466: a029         	j	0x11470 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x78>
   11468: 1b7d         	addi	s6, s6, -0x1
   1146a: 04e1         	addi	s1, s1, 0x18
   1146c: 000b0c63     	beqz	s6, 0x11484 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x8c>
   11470: ff84b583     	ld	a1, -0x8(s1)
   11474: d9f5         	beqz	a1, 0x11468 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x70>
   11476: 6088         	ld	a0, 0x0(s1)
   11478: 4605         	li	a2, 0x1
   1147a: 00000097     	auipc	ra, 0x0
   1147e: 68e080e7     	jalr	0x68e(ra) <__rust_dealloc>
   11482: b7dd         	j	0x11468 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x70>
   11484: 030ab583     	ld	a1, 0x30(s5)
   11488: cd81         	beqz	a1, 0x114a0 <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0xa8>
   1148a: 038ab503     	ld	a0, 0x38(s5)
   1148e: 00359613     	slli	a2, a1, 0x3
   11492: 0596         	slli	a1, a1, 0x5
   11494: 8d91         	sub	a1, a1, a2
   11496: 4621         	li	a2, 0x8
   11498: 00000097     	auipc	ra, 0x0
   1149c: 670080e7     	jalr	0x670(ra) <__rust_dealloc>
   114a0: 048ab583     	ld	a1, 0x48(s5)
   114a4: dda5         	beqz	a1, 0x1141c <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x24>
   114a6: 050ab503     	ld	a0, 0x50(s5)
   114aa: 058e         	slli	a1, a1, 0x3
   114ac: 4621         	li	a2, 0x8
   114ae: 00000097     	auipc	ra, 0x0
   114b2: 65a080e7     	jalr	0x65a(ra) <__rust_dealloc>
   114b6: b79d         	j	0x1141c <_ZN70_$LT$alloc..vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..ops..drop..Drop$GT$4drop17ha7d2f10f2ccc1b0dE+0x24>
   114b8: 70e2         	ld	ra, 0x38(sp)
   114ba: 7442         	ld	s0, 0x30(sp)
   114bc: 74a2         	ld	s1, 0x28(sp)
   114be: 7902         	ld	s2, 0x20(sp)
   114c0: 69e2         	ld	s3, 0x18(sp)
   114c2: 6a42         	ld	s4, 0x10(sp)
   114c4: 6aa2         	ld	s5, 0x8(sp)
   114c6: 6b02         	ld	s6, 0x0(sp)
   114c8: 6121         	addi	sp, sp, 0x40
   114ca: 8082         	ret

00000000000114cc <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934>:
   114cc: 7159         	addi	sp, sp, -0x70
   114ce: f486         	sd	ra, 0x68(sp)
   114d0: f0a2         	sd	s0, 0x60(sp)
   114d2: eca6         	sd	s1, 0x58(sp)
   114d4: e8ca         	sd	s2, 0x50(sp)
   114d6: e4ce         	sd	s3, 0x48(sp)
   114d8: e0d2         	sd	s4, 0x40(sp)
   114da: fc56         	sd	s5, 0x38(sp)
   114dc: f85a         	sd	s6, 0x30(sp)
   114de: f45e         	sd	s7, 0x28(sp)
   114e0: f062         	sd	s8, 0x20(sp)
   114e2: ec66         	sd	s9, 0x18(sp)
   114e4: e86a         	sd	s10, 0x10(sp)
   114e6: e46e         	sd	s11, 0x8(sp)
   114e8: 1880         	addi	s0, sp, 0x70
   114ea: 89aa         	mv	s3, a0
   114ec: 04154503     	lbu	a0, 0x41(a0)
   114f0: 14051163     	bnez	a0, 0x11632 <.Lpcrel_hi3+0x30>
   114f4: 0209bb83     	ld	s7, 0x20(s3)
   114f8: 0289bb03     	ld	s6, 0x28(s3)
   114fc: 0109ba83     	ld	s5, 0x10(s3)
   11500: 117b6d63     	bltu	s6, s7, 0x1161a <.Lpcrel_hi3+0x18>
   11504: 0189bc03     	ld	s8, 0x18(s3)
   11508: 116c6963     	bltu	s8, s6, 0x1161a <.Lpcrel_hi3+0x18>
   1150c: 0389c903     	lbu	s2, 0x38(s3)
   11510: 01298533     	add	a0, s3, s2
   11514: 02f54483     	lbu	s1, 0x2f(a0)
   11518: 4511         	li	a0, 0x4
   1151a: 09256663     	bltu	a0, s2, 0x115a6 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xda>
   1151e: 03098a13     	addi	s4, s3, 0x30
   11522: 41600cb3     	neg	s9, s6
   11526: 4d3d         	li	s10, 0xf
   11528: a019         	j	0x1152e <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x62>
   1152a: 0f7b6863     	bltu	s6, s7, 0x1161a <.Lpcrel_hi3+0x18>
   1152e: 417b0633     	sub	a2, s6, s7
   11532: 017a85b3     	add	a1, s5, s7
   11536: 02cd6063     	bltu	s10, a2, 0x11556 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x8a>
   1153a: 0d7b0e63     	beq	s6, s7, 0x11616 <.Lpcrel_hi3+0x14>
   1153e: 4501         	li	a0, 0x0
   11540: 017c8633     	add	a2, s9, s7
   11544: 0005c683     	lbu	a3, 0x0(a1)
   11548: 02968763     	beq	a3, s1, 0x11576 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xaa>
   1154c: 157d         	addi	a0, a0, -0x1
   1154e: 0585         	addi	a1, a1, 0x1
   11550: fea61ae3     	bne	a2, a0, 0x11544 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x78>
   11554: a0c9         	j	0x11616 <.Lpcrel_hi3+0x14>
   11556: 8526         	mv	a0, s1
   11558: 00002097     	auipc	ra, 0x2
   1155c: 35e080e7     	jalr	0x35e(ra) <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E>
   11560: 4605         	li	a2, 0x1
   11562: 0ec51b63     	bne	a0, a2, 0x11658 <.Lpcrel_hi3+0x56>
   11566: 95de         	add	a1, a1, s7
   11568: 00158b93     	addi	s7, a1, 0x1
   1156c: 0379b023     	sd	s7, 0x20(s3)
   11570: 012bfc63     	bgeu	s7, s2, 0x11588 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xbc>
   11574: bf5d         	j	0x1152a <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x5e>
   11576: 40a005b3     	neg	a1, a0
   1157a: 95de         	add	a1, a1, s7
   1157c: 00158b93     	addi	s7, a1, 0x1
   11580: 0379b023     	sd	s7, 0x20(s3)
   11584: fb2be3e3     	bltu	s7, s2, 0x1152a <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x5e>
   11588: fb7c61e3     	bltu	s8, s7, 0x1152a <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x5e>
   1158c: 412b8db3     	sub	s11, s7, s2
   11590: 01ba8533     	add	a0, s5, s11
   11594: 85d2         	mv	a1, s4
   11596: 864a         	mv	a2, s2
   11598: 00003097     	auipc	ra, 0x3
   1159c: 04e080e7     	jalr	0x4e(ra) <memcmp>
   115a0: 2501         	sext.w	a0, a0
   115a2: f541         	bnez	a0, 0x1152a <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x5e>
   115a4: a875         	j	0x11660 <.Lpcrel_hi3+0x5e>
   115a6: 41600a33     	neg	s4, s6
   115aa: 4cc1         	li	s9, 0x10
   115ac: 4d05         	li	s10, 0x1
   115ae: a019         	j	0x115b4 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xe8>
   115b0: 077b6563     	bltu	s6, s7, 0x1161a <.Lpcrel_hi3+0x18>
   115b4: 417b0633     	sub	a2, s6, s7
   115b8: 017a85b3     	add	a1, s5, s7
   115bc: 03967063     	bgeu	a2, s9, 0x115dc <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x110>
   115c0: 057b0b63     	beq	s6, s7, 0x11616 <.Lpcrel_hi3+0x14>
   115c4: 4501         	li	a0, 0x0
   115c6: 017a0633     	add	a2, s4, s7
   115ca: 0005c683     	lbu	a3, 0x0(a1)
   115ce: 00968f63     	beq	a3, s1, 0x115ec <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x120>
   115d2: 157d         	addi	a0, a0, -0x1
   115d4: 0585         	addi	a1, a1, 0x1
   115d6: fea61ae3     	bne	a2, a0, 0x115ca <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xfe>
   115da: a835         	j	0x11616 <.Lpcrel_hi3+0x14>
   115dc: 8526         	mv	a0, s1
   115de: 00002097     	auipc	ra, 0x2
   115e2: 2d8080e7     	jalr	0x2d8(ra) <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E>
   115e6: 01a50563     	beq	a0, s10, 0x115f0 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0x124>
   115ea: a0bd         	j	0x11658 <.Lpcrel_hi3+0x56>
   115ec: 40a005b3     	neg	a1, a0
   115f0: 95de         	add	a1, a1, s7
   115f2: 00158b93     	addi	s7, a1, 0x1
   115f6: 0379b023     	sd	s7, 0x20(s3)
   115fa: fb2bebe3     	bltu	s7, s2, 0x115b0 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xe4>
   115fe: fb7c69e3     	bltu	s8, s7, 0x115b0 <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934+0xe4>

0000000000011602 <.Lpcrel_hi3>:
   11602: 00004517     	auipc	a0, 0x4
   11606: 05e50613     	addi	a2, a0, 0x5e
   1160a: 4591         	li	a1, 0x4
   1160c: 854a         	mv	a0, s2
   1160e: 00002097     	auipc	ra, 0x2
   11612: 37a080e7     	jalr	0x37a(ra) <_ZN4core5slice5index24slice_end_index_len_fail17h6d0be8bee959f757E>
   11616: 0369b023     	sd	s6, 0x20(s3)
   1161a: 0409c603     	lbu	a2, 0x40(s3)
   1161e: 0009b503     	ld	a0, 0x0(s3)
   11622: 0089b583     	ld	a1, 0x8(s3)
   11626: 4685         	li	a3, 0x1
   11628: 04d980a3     	sb	a3, 0x41(s3)
   1162c: e609         	bnez	a2, 0x11636 <.Lpcrel_hi3+0x34>
   1162e: 00a59463     	bne	a1, a0, 0x11636 <.Lpcrel_hi3+0x34>
   11632: 4501         	li	a0, 0x0
   11634: a019         	j	0x1163a <.Lpcrel_hi3+0x38>
   11636: 8d89         	sub	a1, a1, a0
   11638: 9556         	add	a0, a0, s5
   1163a: 70a6         	ld	ra, 0x68(sp)
   1163c: 7406         	ld	s0, 0x60(sp)
   1163e: 64e6         	ld	s1, 0x58(sp)
   11640: 6946         	ld	s2, 0x50(sp)
   11642: 69a6         	ld	s3, 0x48(sp)
   11644: 6a06         	ld	s4, 0x40(sp)
   11646: 7ae2         	ld	s5, 0x38(sp)
   11648: 7b42         	ld	s6, 0x30(sp)
   1164a: 7ba2         	ld	s7, 0x28(sp)
   1164c: 7c02         	ld	s8, 0x20(sp)
   1164e: 6ce2         	ld	s9, 0x18(sp)
   11650: 6d42         	ld	s10, 0x10(sp)
   11652: 6da2         	ld	s11, 0x8(sp)
   11654: 6165         	addi	sp, sp, 0x70
   11656: 8082         	ret
   11658: 8905         	andi	a0, a0, 0x1
   1165a: 0369b023     	sd	s6, 0x20(s3)
   1165e: dd55         	beqz	a0, 0x1161a <.Lpcrel_hi3+0x18>
   11660: 0009b503     	ld	a0, 0x0(s3)
   11664: 40ad85b3     	sub	a1, s11, a0
   11668: 9556         	add	a0, a0, s5
   1166a: 0179b023     	sd	s7, 0x0(s3)
   1166e: b7f1         	j	0x1163a <.Lpcrel_hi3+0x38>

0000000000011670 <_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h5a536bd7b0ac802eE>:
   11670: 7155         	addi	sp, sp, -0xd0
   11672: e586         	sd	ra, 0xc8(sp)
   11674: e1a2         	sd	s0, 0xc0(sp)
   11676: fd26         	sd	s1, 0xb8(sp)
   11678: f94a         	sd	s2, 0xb0(sp)
   1167a: f54e         	sd	s3, 0xa8(sp)
   1167c: f152         	sd	s4, 0xa0(sp)
   1167e: ed56         	sd	s5, 0x98(sp)
   11680: e95a         	sd	s6, 0x90(sp)
   11682: e55e         	sd	s7, 0x88(sp)
   11684: e162         	sd	s8, 0x80(sp)
   11686: fce6         	sd	s9, 0x78(sp)
   11688: f8ea         	sd	s10, 0x70(sp)
   1168a: f4ee         	sd	s11, 0x68(sp)
   1168c: 0980         	addi	s0, sp, 0xd0
   1168e: 0ac58463     	beq	a1, a2, 0x11736 <.Lpcrel_hi4+0x44>
   11692: 6584         	ld	s1, 0x8(a1)
   11694: 01058c13     	addi	s8, a1, 0x10
   11698: 85e2         	mv	a1, s8
   1169a: d8f5         	beqz	s1, 0x1168e <_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h5a536bd7b0ac802eE+0x1e>
   1169c: 8936         	mv	s2, a3
   1169e: 8ab2         	mv	s5, a2
   116a0: 8b2a         	mv	s6, a0
   116a2: ff0c3983     	ld	s3, -0x10(s8)
   116a6: f8043023     	sd	zero, -0x80(s0)
   116aa: f8840b93     	addi	s7, s0, -0x78
   116ae: 4505         	li	a0, 0x1
   116b0: f8a43423     	sd	a0, -0x78(s0)
   116b4: f8043823     	sd	zero, -0x70(s0)
   116b8: f8040513     	addi	a0, s0, -0x80
   116bc: 4685         	li	a3, 0x1
   116be: 4705         	li	a4, 0x1
   116c0: 4581         	li	a1, 0x0
   116c2: 8626         	mv	a2, s1
   116c4: 00000097     	auipc	ra, 0x0
   116c8: c50080e7     	jalr	-0x3b0(ra) <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E>
   116cc: f9043c83     	ld	s9, -0x70(s0)
   116d0: f8843a03     	ld	s4, -0x78(s0)
   116d4: f8043d03     	ld	s10, -0x80(s0)
   116d8: 019a0533     	add	a0, s4, s9
   116dc: 85ce         	mv	a1, s3
   116de: 8626         	mv	a2, s1
   116e0: 00003097     	auipc	ra, 0x3
   116e4: e38080e7     	jalr	-0x1c8(ra) <memcpy>
   116e8: 94e6         	add	s1, s1, s9
   116ea: f8943823     	sd	s1, -0x70(s0)
   116ee: 01a49e63     	bne	s1, s10, 0x1170a <.Lpcrel_hi4+0x18>

00000000000116f2 <.Lpcrel_hi4>:
   116f2: 00004517     	auipc	a0, 0x4
   116f6: 98e50593     	addi	a1, a0, -0x672
   116fa: f8040513     	addi	a0, s0, -0x80
   116fe: 00001097     	auipc	ra, 0x1
   11702: f5c080e7     	jalr	-0xa4(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE>
   11706: f8843a03     	ld	s4, -0x78(s0)
   1170a: 9a26         	add	s4, s4, s1
   1170c: 000a0023     	sb	zero, 0x0(s4)
   11710: 0485         	addi	s1, s1, 0x1
   11712: 000bb503     	ld	a0, 0x0(s7)
   11716: f8943823     	sd	s1, -0x70(s0)
   1171a: f8043483     	ld	s1, -0x80(s0)
   1171e: 008bb583     	ld	a1, 0x8(s7)
   11722: f6a43023     	sd	a0, -0xa0(s0)
   11726: 557d         	li	a0, -0x1
   11728: 03f51613     	slli	a2, a0, 0x3f
   1172c: f6b43423     	sd	a1, -0x98(s0)
   11730: 855a         	mv	a0, s6
   11732: 02c49763     	bne	s1, a2, 0x11760 <.Lpcrel_hi5>
   11736: 00053023     	sd	zero, 0x0(a0)
   1173a: 45a1         	li	a1, 0x8
   1173c: e50c         	sd	a1, 0x8(a0)
   1173e: 00053823     	sd	zero, 0x10(a0)
   11742: 60ae         	ld	ra, 0xc8(sp)
   11744: 640e         	ld	s0, 0xc0(sp)
   11746: 74ea         	ld	s1, 0xb8(sp)
   11748: 794a         	ld	s2, 0xb0(sp)
   1174a: 79aa         	ld	s3, 0xa8(sp)
   1174c: 7a0a         	ld	s4, 0xa0(sp)
   1174e: 6aea         	ld	s5, 0x98(sp)
   11750: 6b4a         	ld	s6, 0x90(sp)
   11752: 6baa         	ld	s7, 0x88(sp)
   11754: 6c0a         	ld	s8, 0x80(sp)
   11756: 7ce6         	ld	s9, 0x78(sp)
   11758: 7d46         	ld	s10, 0x70(sp)
   1175a: 7da6         	ld	s11, 0x68(sp)
   1175c: 6169         	addi	sp, sp, 0xd0
   1175e: 8082         	ret

0000000000011760 <.Lpcrel_hi5>:
   11760: 0000e517     	auipc	a0, 0xe
   11764: 9c954003     	lbu	zero, -0x637(a0)
   11768: 06000513     	li	a0, 0x60
   1176c: 45a1         	li	a1, 0x8
   1176e: 00000097     	auipc	ra, 0x0
   11772: 376080e7     	jalr	0x376(ra) <__rust_alloc>
   11776: 12050d63     	beqz	a0, 0x118b0 <.Lpcrel_hi6+0x118>
   1177a: f6043583     	ld	a1, -0xa0(s0)
   1177e: f6843603     	ld	a2, -0x98(s0)
   11782: e104         	sd	s1, 0x0(a0)
   11784: e50c         	sd	a1, 0x8(a0)
   11786: e910         	sd	a2, 0x10(a0)
   11788: 4591         	li	a1, 0x4
   1178a: f4b43423     	sd	a1, -0xb8(s0)
   1178e: f4a43823     	sd	a0, -0xb0(s0)
   11792: 4c85         	li	s9, 0x1
   11794: f5943c23     	sd	s9, -0xa8(s0)

0000000000011798 <.Lpcrel_hi6>:
   11798: 00004597     	auipc	a1, 0x4
   1179c: 8e858913     	addi	s2, a1, -0x718
   117a0: 5d7d         	li	s10, -0x1
   117a2: 1d7e         	slli	s10, s10, 0x3f
   117a4: 4985         	li	s3, 0x1
   117a6: 85da         	mv	a1, s6
   117a8: 8656         	mv	a2, s5
   117aa: 86e2         	mv	a3, s8
   117ac: 0ec68863     	beq	a3, a2, 0x1189c <.Lpcrel_hi6+0x104>
   117b0: 6684         	ld	s1, 0x8(a3)
   117b2: 01068c13     	addi	s8, a3, 0x10
   117b6: 86e2         	mv	a3, s8
   117b8: d8f5         	beqz	s1, 0x117ac <.Lpcrel_hi6+0x14>
   117ba: f4a43023     	sd	a0, -0xc0(s0)
   117be: ff0c3503     	ld	a0, -0x10(s8)
   117c2: f2a43c23     	sd	a0, -0xc8(s0)
   117c6: f8043023     	sd	zero, -0x80(s0)
   117ca: f9943423     	sd	s9, -0x78(s0)
   117ce: f8043823     	sd	zero, -0x70(s0)
   117d2: f8040513     	addi	a0, s0, -0x80
   117d6: 4685         	li	a3, 0x1
   117d8: 4705         	li	a4, 0x1
   117da: 4581         	li	a1, 0x0
   117dc: 8626         	mv	a2, s1
   117de: 00000097     	auipc	ra, 0x0
   117e2: b36080e7     	jalr	-0x4ca(ra) <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E>
   117e6: f9043a03     	ld	s4, -0x70(s0)
   117ea: f8843d83     	ld	s11, -0x78(s0)
   117ee: f8043503     	ld	a0, -0x80(s0)
   117f2: f2a43823     	sd	a0, -0xd0(s0)
   117f6: 014d8533     	add	a0, s11, s4
   117fa: f3843583     	ld	a1, -0xc8(s0)
   117fe: 8626         	mv	a2, s1
   11800: 00003097     	auipc	ra, 0x3
   11804: d18080e7     	jalr	-0x2e8(ra) <memcpy>
   11808: 94d2         	add	s1, s1, s4
   1180a: f8943823     	sd	s1, -0x70(s0)
   1180e: f3043503     	ld	a0, -0xd0(s0)
   11812: 00a49b63     	bne	s1, a0, 0x11828 <.Lpcrel_hi6+0x90>
   11816: f8040513     	addi	a0, s0, -0x80
   1181a: 85ca         	mv	a1, s2
   1181c: 00001097     	auipc	ra, 0x1
   11820: e3e080e7     	jalr	-0x1c2(ra) <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE>
   11824: f8843d83     	ld	s11, -0x78(s0)
   11828: 9da6         	add	s11, s11, s1
   1182a: 000d8023     	sb	zero, 0x0(s11)
   1182e: 0485         	addi	s1, s1, 0x1
   11830: f8943823     	sd	s1, -0x70(s0)
   11834: f8043483     	ld	s1, -0x80(s0)
   11838: 000bb503     	ld	a0, 0x0(s7)
   1183c: 008bb583     	ld	a1, 0x8(s7)
   11840: f6a43823     	sd	a0, -0x90(s0)
   11844: f6b43c23     	sd	a1, -0x88(s0)
   11848: 85da         	mv	a1, s6
   1184a: f4043503     	ld	a0, -0xc0(s0)
   1184e: 05a48763     	beq	s1, s10, 0x1189c <.Lpcrel_hi6+0x104>
   11852: 8656         	mv	a2, s5
   11854: f4843683     	ld	a3, -0xb8(s0)
   11858: 02d98363     	beq	s3, a3, 0x1187e <.Lpcrel_hi6+0xe6>
   1185c: 00399693     	slli	a3, s3, 0x3
   11860: 00599713     	slli	a4, s3, 0x5
   11864: 8f15         	sub	a4, a4, a3
   11866: 972a         	add	a4, a4, a0
   11868: e304         	sd	s1, 0x0(a4)
   1186a: f7043683     	ld	a3, -0x90(s0)
   1186e: e714         	sd	a3, 0x8(a4)
   11870: f7843683     	ld	a3, -0x88(s0)
   11874: eb14         	sd	a3, 0x10(a4)
   11876: 0985         	addi	s3, s3, 0x1
   11878: f5343c23     	sd	s3, -0xa8(s0)
   1187c: b73d         	j	0x117aa <.Lpcrel_hi6+0x12>
   1187e: f4840513     	addi	a0, s0, -0xb8
   11882: 4605         	li	a2, 0x1
   11884: 46a1         	li	a3, 0x8
   11886: 4761         	li	a4, 0x18
   11888: 85ce         	mv	a1, s3
   1188a: 00000097     	auipc	ra, 0x0
   1188e: a8a080e7     	jalr	-0x576(ra) <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E>
   11892: 8656         	mv	a2, s5
   11894: 85da         	mv	a1, s6
   11896: f5043503     	ld	a0, -0xb0(s0)
   1189a: b7c9         	j	0x1185c <.Lpcrel_hi6+0xc4>
   1189c: f5843503     	ld	a0, -0xa8(s0)
   118a0: f5043603     	ld	a2, -0xb0(s0)
   118a4: f4843683     	ld	a3, -0xb8(s0)
   118a8: e988         	sd	a0, 0x10(a1)
   118aa: e590         	sd	a2, 0x8(a1)
   118ac: e194         	sd	a3, 0x0(a1)
   118ae: bd51         	j	0x11742 <.Lpcrel_hi4+0x50>
   118b0: 4521         	li	a0, 0x8
   118b2: 06000593     	li	a1, 0x60
   118b6: 864a         	mv	a2, s2
   118b8: 00001097     	auipc	ra, 0x1
   118bc: ec4080e7     	jalr	-0x13c(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

00000000000118c0 <_ZN98_$LT$alloc..vec..Vec$LT$T$GT$$u20$as$u20$alloc..vec..spec_from_iter..SpecFromIter$LT$T$C$I$GT$$GT$9from_iter17h5b91d505f4d5c4f4E>:
   118c0: 7135         	addi	sp, sp, -0xa0
   118c2: ed06         	sd	ra, 0x98(sp)
   118c4: e922         	sd	s0, 0x90(sp)
   118c6: e526         	sd	s1, 0x88(sp)
   118c8: e14a         	sd	s2, 0x80(sp)
   118ca: fcce         	sd	s3, 0x78(sp)
   118cc: f8d2         	sd	s4, 0x70(sp)
   118ce: f4d6         	sd	s5, 0x68(sp)
   118d0: f0da         	sd	s6, 0x60(sp)
   118d2: 1100         	addi	s0, sp, 0xa0
   118d4: 8ab2         	mv	s5, a2
   118d6: 8a2e         	mv	s4, a1
   118d8: 892a         	mv	s2, a0
   118da: 852e         	mv	a0, a1
   118dc: 00000097     	auipc	ra, 0x0
   118e0: bf0080e7     	jalr	-0x410(ra) <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934>
   118e4: cd55         	beqz	a0, 0x119a0 <.Lpcrel_hi7+0xb6>
   118e6: 84aa         	mv	s1, a0
   118e8: 8b2e         	mv	s6, a1

00000000000118ea <.Lpcrel_hi7>:
   118ea: 0000e517     	auipc	a0, 0xe
   118ee: 83f54003     	lbu	zero, -0x7c1(a0)
   118f2: 04000513     	li	a0, 0x40
   118f6: 45a1         	li	a1, 0x8
   118f8: 00000097     	auipc	ra, 0x0
   118fc: 1ec080e7     	jalr	0x1ec(ra) <__rust_alloc>
   11900: c169         	beqz	a0, 0x119c2 <.Lpcrel_hi7+0xd8>
   11902: 89aa         	mv	s3, a0
   11904: e104         	sd	s1, 0x0(a0)
   11906: 01653423     	sd	s6, 0x8(a0)
   1190a: 4511         	li	a0, 0x4
   1190c: f6a43023     	sd	a0, -0xa0(s0)
   11910: f7343423     	sd	s3, -0x98(s0)
   11914: 4a85         	li	s5, 0x1
   11916: f7543823     	sd	s5, -0x90(s0)
   1191a: f7840513     	addi	a0, s0, -0x88
   1191e: 04800613     	li	a2, 0x48
   11922: 85d2         	mv	a1, s4
   11924: 00003097     	auipc	ra, 0x3
   11928: bf4080e7     	jalr	-0x40c(ra) <memcpy>
   1192c: f7840513     	addi	a0, s0, -0x88
   11930: 00000097     	auipc	ra, 0x0
   11934: b9c080e7     	jalr	-0x464(ra) <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934>
   11938: c539         	beqz	a0, 0x11986 <.Lpcrel_hi7+0x9c>
   1193a: 4481         	li	s1, 0x0
   1193c: a005         	j	0x1195c <.Lpcrel_hi7+0x72>
   1193e: 00998633     	add	a2, s3, s1
   11942: ea08         	sd	a0, 0x10(a2)
   11944: ee0c         	sd	a1, 0x18(a2)
   11946: 0a85         	addi	s5, s5, 0x1
   11948: f7543823     	sd	s5, -0x90(s0)
   1194c: f7840513     	addi	a0, s0, -0x88
   11950: 00000097     	auipc	ra, 0x0
   11954: b7c080e7     	jalr	-0x484(ra) <_ZN90_$LT$core..str..iter..Split$LT$P$GT$$u20$as$u20$core..iter..traits..iterator..Iterator$GT$4next17habe8df60c1e24bd2E.llvm.9545185464230356934>
   11958: 04c1         	addi	s1, s1, 0x10
   1195a: c515         	beqz	a0, 0x11986 <.Lpcrel_hi7+0x9c>
   1195c: f6043603     	ld	a2, -0xa0(s0)
   11960: fcca9fe3     	bne	s5, a2, 0x1193e <.Lpcrel_hi7+0x54>
   11964: 89aa         	mv	s3, a0
   11966: f6040513     	addi	a0, s0, -0xa0
   1196a: 4605         	li	a2, 0x1
   1196c: 46a1         	li	a3, 0x8
   1196e: 4741         	li	a4, 0x10
   11970: 8a2e         	mv	s4, a1
   11972: 85d6         	mv	a1, s5
   11974: 00000097     	auipc	ra, 0x0
   11978: 9a0080e7     	jalr	-0x660(ra) <_ZN5alloc7raw_vec20RawVecInner$LT$A$GT$7reserve21do_reserve_and_handle17hba046d1e86cc0397E>
   1197c: 85d2         	mv	a1, s4
   1197e: 854e         	mv	a0, s3
   11980: f6843983     	ld	s3, -0x98(s0)
   11984: bf6d         	j	0x1193e <.Lpcrel_hi7+0x54>
   11986: f7043503     	ld	a0, -0x90(s0)
   1198a: f6843583     	ld	a1, -0x98(s0)
   1198e: f6043603     	ld	a2, -0xa0(s0)
   11992: 00a93823     	sd	a0, 0x10(s2)
   11996: 00b93423     	sd	a1, 0x8(s2)
   1199a: 00c93023     	sd	a2, 0x0(s2)
   1199e: a801         	j	0x119ae <.Lpcrel_hi7+0xc4>
   119a0: 00093023     	sd	zero, 0x0(s2)
   119a4: 4521         	li	a0, 0x8
   119a6: 00a93423     	sd	a0, 0x8(s2)
   119aa: 00093823     	sd	zero, 0x10(s2)
   119ae: 60ea         	ld	ra, 0x98(sp)
   119b0: 644a         	ld	s0, 0x90(sp)
   119b2: 64aa         	ld	s1, 0x88(sp)
   119b4: 690a         	ld	s2, 0x80(sp)
   119b6: 79e6         	ld	s3, 0x78(sp)
   119b8: 7a46         	ld	s4, 0x70(sp)
   119ba: 7aa6         	ld	s5, 0x68(sp)
   119bc: 7b06         	ld	s6, 0x60(sp)
   119be: 610d         	addi	sp, sp, 0xa0
   119c0: 8082         	ret
   119c2: 4521         	li	a0, 0x8
   119c4: 04000593     	li	a1, 0x40
   119c8: 8656         	mv	a2, s5
   119ca: 00001097     	auipc	ra, 0x1
   119ce: db2080e7     	jalr	-0x24e(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

00000000000119d2 <__rust_alloc_error_handler>:
   119d2: 00000317     	auipc	t1, 0x0
   119d6: 22030067     	jr	0x220(t1) <__rg_oom>

00000000000119da <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17h002ec60b2de15ccdE>:
   119da: 1141         	addi	sp, sp, -0x10
   119dc: e406         	sd	ra, 0x8(sp)
   119de: e022         	sd	s0, 0x0(sp)
   119e0: 0800         	addi	s0, sp, 0x10
   119e2: 0245e603     	lwu	a2, 0x24(a1)
   119e6: 01067693     	andi	a3, a2, 0x10
   119ea: ea99         	bnez	a3, 0x11a00 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17h002ec60b2de15ccdE+0x26>
   119ec: 02067613     	andi	a2, a2, 0x20
   119f0: ee19         	bnez	a2, 0x11a0e <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..Debug$u20$for$u20$usize$GT$3fmt17h002ec60b2de15ccdE+0x34>
   119f2: 60a2         	ld	ra, 0x8(sp)
   119f4: 6402         	ld	s0, 0x0(sp)
   119f6: 0141         	addi	sp, sp, 0x10
   119f8: 00003317     	auipc	t1, 0x3
   119fc: 88230067     	jr	-0x77e(t1) <_ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17hb7c5519abaf1703fE>
   11a00: 60a2         	ld	ra, 0x8(sp)
   11a02: 6402         	ld	s0, 0x0(sp)
   11a04: 0141         	addi	sp, sp, 0x10
   11a06: 00002317     	auipc	t1, 0x2
   11a0a: 4dc30067     	jr	0x4dc(t1) <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E>
   11a0e: 60a2         	ld	ra, 0x8(sp)
   11a10: 6402         	ld	s0, 0x0(sp)
   11a12: 0141         	addi	sp, sp, 0x10
   11a14: 00002317     	auipc	t1, 0x2
   11a18: 52830067     	jr	0x528(t1) <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE>

0000000000011a1c <_ZN64_$LT$core..alloc..layout..Layout$u20$as$u20$core..fmt..Debug$GT$3fmt17h7611f9a4df4a3292E>:
   11a1c: 7179         	addi	sp, sp, -0x30
   11a1e: f406         	sd	ra, 0x28(sp)
   11a20: f022         	sd	s0, 0x20(sp)
   11a22: 1800         	addi	s0, sp, 0x30
   11a24: 82ae         	mv	t0, a1
   11a26: 00850793     	addi	a5, a0, 0x8
   11a2a: fea43423     	sd	a0, -0x18(s0)

0000000000011a2e <.Lpcrel_hi4>:
   11a2e: 00004517     	auipc	a0, 0x4
   11a32: c6a50513     	addi	a0, a0, -0x396
   11a36: e82a         	sd	a0, 0x10(sp)
   11a38: fe840513     	addi	a0, s0, -0x18
   11a3c: e42a         	sd	a0, 0x8(sp)
   11a3e: 4515         	li	a0, 0x5

0000000000011a40 <.Lpcrel_hi5>:
   11a40: 00004597     	auipc	a1, 0x4
   11a44: c7858593     	addi	a1, a1, -0x388

0000000000011a48 <.Lpcrel_hi6>:
   11a48: 00004617     	auipc	a2, 0x4
   11a4c: a4060693     	addi	a3, a2, -0x5c0

0000000000011a50 <.Lpcrel_hi7>:
   11a50: 00004617     	auipc	a2, 0x4
   11a54: c2860813     	addi	a6, a2, -0x3d8

0000000000011a58 <.Lpcrel_hi8>:
   11a58: 00004617     	auipc	a2, 0x4
   11a5c: c6660893     	addi	a7, a2, -0x39a
   11a60: 4619         	li	a2, 0x6
   11a62: 4711         	li	a4, 0x4
   11a64: e02a         	sd	a0, 0x0(sp)
   11a66: 8516         	mv	a0, t0
   11a68: 00002097     	auipc	ra, 0x2
   11a6c: b4c080e7     	jalr	-0x4b4(ra) <_ZN4core3fmt9Formatter26debug_struct_field2_finish17hb31ed29359c11ddfE>
   11a70: 70a2         	ld	ra, 0x28(sp)
   11a72: 7402         	ld	s0, 0x20(sp)
   11a74: 6145         	addi	sp, sp, 0x30
   11a76: 8082         	ret

0000000000011a78 <_ZN64_$LT$core..str..error..Utf8Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h3b1fc894f8d2c1b6E>:
   11a78: 7179         	addi	sp, sp, -0x30
   11a7a: f406         	sd	ra, 0x28(sp)
   11a7c: f022         	sd	s0, 0x20(sp)
   11a7e: 1800         	addi	s0, sp, 0x30
   11a80: 82ae         	mv	t0, a1
   11a82: 87aa         	mv	a5, a0
   11a84: 0521         	addi	a0, a0, 0x8
   11a86: fea43423     	sd	a0, -0x18(s0)

0000000000011a8a <.Lpcrel_hi9>:
   11a8a: 00004517     	auipc	a0, 0x4
   11a8e: c3e50513     	addi	a0, a0, -0x3c2
   11a92: e82a         	sd	a0, 0x10(sp)
   11a94: fe840513     	addi	a0, s0, -0x18
   11a98: e42a         	sd	a0, 0x8(sp)
   11a9a: 4525         	li	a0, 0x9

0000000000011a9c <.Lpcrel_hi10>:
   11a9c: 00004597     	auipc	a1, 0x4
   11aa0: c4c58593     	addi	a1, a1, -0x3b4

0000000000011aa4 <.Lpcrel_hi11>:
   11aa4: 00004617     	auipc	a2, 0x4
   11aa8: c4d60693     	addi	a3, a2, -0x3b3

0000000000011aac <.Lpcrel_hi12>:
   11aac: 00004617     	auipc	a2, 0x4
   11ab0: bcc60813     	addi	a6, a2, -0x434

0000000000011ab4 <.Lpcrel_hi13>:
   11ab4: 00004617     	auipc	a2, 0x4
   11ab8: c4860893     	addi	a7, a2, -0x3b8
   11abc: 4625         	li	a2, 0x9
   11abe: 472d         	li	a4, 0xb
   11ac0: e02a         	sd	a0, 0x0(sp)
   11ac2: 8516         	mv	a0, t0
   11ac4: 00002097     	auipc	ra, 0x2
   11ac8: af0080e7     	jalr	-0x510(ra) <_ZN4core3fmt9Formatter26debug_struct_field2_finish17hb31ed29359c11ddfE>
   11acc: 70a2         	ld	ra, 0x28(sp)
   11ace: 7402         	ld	s0, 0x20(sp)
   11ad0: 6145         	addi	sp, sp, 0x30
   11ad2: 8082         	ret

0000000000011ad4 <_ZN8user_lib4exit17h98d922c79b0ff27aE>:
   11ad4: 1141         	addi	sp, sp, -0x10
   11ad6: e406         	sd	ra, 0x8(sp)
   11ad8: e022         	sd	s0, 0x0(sp)
   11ada: 0800         	addi	s0, sp, 0x10
   11adc: 00000097     	auipc	ra, 0x0
   11ae0: 5e0080e7     	jalr	0x5e0(ra) <_ZN8user_lib7syscall8sys_exit17h3bae8de294916964E>

0000000000011ae4 <__rust_alloc>:
   11ae4: 1141         	addi	sp, sp, -0x10
   11ae6: e406         	sd	ra, 0x8(sp)
   11ae8: e022         	sd	s0, 0x0(sp)
   11aea: 0800         	addi	s0, sp, 0x10

0000000000011aec <.Lpcrel_hi40>:
   11aec: 0000d617     	auipc	a2, 0xd
   11af0: 51460613     	addi	a2, a2, 0x514
   11af4: 86aa         	mv	a3, a0
   11af6: 8532         	mv	a0, a2
   11af8: 8636         	mv	a2, a3
   11afa: 60a2         	ld	ra, 0x8(sp)
   11afc: 6402         	ld	s0, 0x0(sp)
   11afe: 0141         	addi	sp, sp, 0x10
   11b00: 00001317     	auipc	t1, 0x1
   11b04: a8830067     	jr	-0x578(t1) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE>

0000000000011b08 <__rust_dealloc>:
   11b08: 1141         	addi	sp, sp, -0x10
   11b0a: e406         	sd	ra, 0x8(sp)
   11b0c: e022         	sd	s0, 0x0(sp)
   11b0e: 0800         	addi	s0, sp, 0x10

0000000000011b10 <.Lpcrel_hi41>:
   11b10: 0000d697     	auipc	a3, 0xd
   11b14: 4f068693     	addi	a3, a3, 0x4f0
   11b18: 872e         	mv	a4, a1
   11b1a: 85aa         	mv	a1, a0
   11b1c: 8536         	mv	a0, a3
   11b1e: 86ba         	mv	a3, a4
   11b20: 60a2         	ld	ra, 0x8(sp)
   11b22: 6402         	ld	s0, 0x0(sp)
   11b24: 0141         	addi	sp, sp, 0x10
   11b26: 00001317     	auipc	t1, 0x1
   11b2a: ab030067     	jr	-0x550(t1) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7dealloc17h6485ec3a8003afbfE>

0000000000011b2e <__rust_realloc>:
   11b2e: 7179         	addi	sp, sp, -0x30
   11b30: f406         	sd	ra, 0x28(sp)
   11b32: f022         	sd	s0, 0x20(sp)
   11b34: ec26         	sd	s1, 0x18(sp)
   11b36: e84a         	sd	s2, 0x10(sp)
   11b38: e44e         	sd	s3, 0x8(sp)
   11b3a: e052         	sd	s4, 0x0(sp)
   11b3c: 1800         	addi	s0, sp, 0x30
   11b3e: 84b6         	mv	s1, a3
   11b40: 8932         	mv	s2, a2
   11b42: 8a2e         	mv	s4, a1
   11b44: 89aa         	mv	s3, a0

0000000000011b46 <.Lpcrel_hi42>:
   11b46: 0000d517     	auipc	a0, 0xd
   11b4a: 4ba50513     	addi	a0, a0, 0x4ba
   11b4e: 85b2         	mv	a1, a2
   11b50: 8636         	mv	a2, a3
   11b52: 00001097     	auipc	ra, 0x1
   11b56: a36080e7     	jalr	-0x5ca(ra) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE>
   11b5a: c51d         	beqz	a0, 0x11b88 <.Lpcrel_hi43+0x18>
   11b5c: 8652         	mv	a2, s4
   11b5e: 009a6363     	bltu	s4, s1, 0x11b64 <.Lpcrel_hi42+0x1e>
   11b62: 8626         	mv	a2, s1
   11b64: 84aa         	mv	s1, a0
   11b66: 85ce         	mv	a1, s3
   11b68: 00003097     	auipc	ra, 0x3
   11b6c: 9b0080e7     	jalr	-0x650(ra) <memcpy>

0000000000011b70 <.Lpcrel_hi43>:
   11b70: 0000d517     	auipc	a0, 0xd
   11b74: 49050513     	addi	a0, a0, 0x490
   11b78: 85ce         	mv	a1, s3
   11b7a: 864a         	mv	a2, s2
   11b7c: 86d2         	mv	a3, s4
   11b7e: 00001097     	auipc	ra, 0x1
   11b82: a58080e7     	jalr	-0x5a8(ra) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7dealloc17h6485ec3a8003afbfE>
   11b86: 8526         	mv	a0, s1
   11b88: 70a2         	ld	ra, 0x28(sp)
   11b8a: 7402         	ld	s0, 0x20(sp)
   11b8c: 64e2         	ld	s1, 0x18(sp)
   11b8e: 6942         	ld	s2, 0x10(sp)
   11b90: 69a2         	ld	s3, 0x8(sp)
   11b92: 6a02         	ld	s4, 0x0(sp)
   11b94: 6145         	addi	sp, sp, 0x30
   11b96: 8082         	ret

0000000000011b98 <_ZN8user_lib18handle_alloc_error17h46e5591cd08cfd5dE>:
   11b98: 711d         	addi	sp, sp, -0x60
   11b9a: ec86         	sd	ra, 0x58(sp)
   11b9c: e8a2         	sd	s0, 0x50(sp)
   11b9e: 1080         	addi	s0, sp, 0x60
   11ba0: faa43023     	sd	a0, -0x60(s0)
   11ba4: fab43423     	sd	a1, -0x58(s0)
   11ba8: fa040513     	addi	a0, s0, -0x60
   11bac: fea43023     	sd	a0, -0x20(s0)

0000000000011bb0 <.Lpcrel_hi45>:
   11bb0: 00000517     	auipc	a0, 0x0
   11bb4: e6c50513     	addi	a0, a0, -0x194
   11bb8: fea43423     	sd	a0, -0x18(s0)

0000000000011bbc <.Lpcrel_hi46>:
   11bbc: 00004517     	auipc	a0, 0x4
   11bc0: c1450513     	addi	a0, a0, -0x3ec
   11bc4: faa43823     	sd	a0, -0x50(s0)
   11bc8: 4505         	li	a0, 0x1
   11bca: faa43c23     	sd	a0, -0x48(s0)
   11bce: fc043823     	sd	zero, -0x30(s0)
   11bd2: fe040593     	addi	a1, s0, -0x20
   11bd6: fcb43023     	sd	a1, -0x40(s0)
   11bda: fca43423     	sd	a0, -0x38(s0)

0000000000011bde <.Lpcrel_hi47>:
   11bde: 00004517     	auipc	a0, 0x4
   11be2: c0250593     	addi	a1, a0, -0x3fe
   11be6: fb040513     	addi	a0, s0, -0x50
   11bea: 00001097     	auipc	ra, 0x1
   11bee: d4e080e7     	jalr	-0x2b2(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

0000000000011bf2 <__rg_oom>:
   11bf2: 1141         	addi	sp, sp, -0x10
   11bf4: e406         	sd	ra, 0x8(sp)
   11bf6: e022         	sd	s0, 0x0(sp)
   11bf8: 0800         	addi	s0, sp, 0x10
   11bfa: 862a         	mv	a2, a0
   11bfc: 852e         	mv	a0, a1
   11bfe: 85b2         	mv	a1, a2
   11c00: 00000097     	auipc	ra, 0x0
   11c04: f98080e7     	jalr	-0x68(ra) <_ZN8user_lib18handle_alloc_error17h46e5591cd08cfd5dE>

0000000000011c08 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h41238fc4770261d9E>:
   11c08: 6110         	ld	a2, 0x0(a0)
   11c0a: 00064683     	lbu	a3, 0x0(a2)
   11c0e: 852e         	mv	a0, a1
   11c10: ca9d         	beqz	a3, 0x11c46 <.Lpcrel_hi0>
   11c12: 1101         	addi	sp, sp, -0x20
   11c14: ec06         	sd	ra, 0x18(sp)
   11c16: e822         	sd	s0, 0x10(sp)
   11c18: 1000         	addi	s0, sp, 0x20
   11c1a: 0605         	addi	a2, a2, 0x1
   11c1c: fec43423     	sd	a2, -0x18(s0)

0000000000011c20 <.Lpcrel_hi1>:
   11c20: 00004597     	auipc	a1, 0x4
   11c24: 85858593     	addi	a1, a1, -0x7a8

0000000000011c28 <.Lpcrel_hi2>:
   11c28: 00004617     	auipc	a2, 0x4
   11c2c: bd060713     	addi	a4, a2, -0x430
   11c30: 4611         	li	a2, 0x4
   11c32: fe840693     	addi	a3, s0, -0x18
   11c36: 00002097     	auipc	ra, 0x2
   11c3a: a4e080e7     	jalr	-0x5b2(ra) <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE>
   11c3e: 60e2         	ld	ra, 0x18(sp)
   11c40: 6442         	ld	s0, 0x10(sp)
   11c42: 6105         	addi	sp, sp, 0x20
   11c44: 8082         	ret

0000000000011c46 <.Lpcrel_hi0>:
   11c46: 00004597     	auipc	a1, 0x4
   11c4a: 83a58593     	addi	a1, a1, -0x7c6
   11c4e: 4611         	li	a2, 0x4
   11c50: 00002317     	auipc	t1, 0x2
   11c54: 94e30067     	jr	-0x6b2(t1) <_ZN57_$LT$core..fmt..Formatter$u20$as$u20$core..fmt..Write$GT$9write_str17h41d63e4bf0c652c7E>

0000000000011c58 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h552f0241ef62fb8fE>:
   11c58: 1141         	addi	sp, sp, -0x10
   11c5a: e406         	sd	ra, 0x8(sp)
   11c5c: e022         	sd	s0, 0x0(sp)
   11c5e: 0800         	addi	s0, sp, 0x10
   11c60: 0245e603     	lwu	a2, 0x24(a1)
   11c64: 6108         	ld	a0, 0x0(a0)
   11c66: 01067693     	andi	a3, a2, 0x10
   11c6a: ea99         	bnez	a3, 0x11c80 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h552f0241ef62fb8fE+0x28>
   11c6c: 02067613     	andi	a2, a2, 0x20
   11c70: ee19         	bnez	a2, 0x11c8e <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h552f0241ef62fb8fE+0x36>
   11c72: 60a2         	ld	ra, 0x8(sp)
   11c74: 6402         	ld	s0, 0x0(sp)
   11c76: 0141         	addi	sp, sp, 0x10
   11c78: 00002317     	auipc	t1, 0x2
   11c7c: 3e030067     	jr	0x3e0(t1) <_ZN4core3fmt3num3imp51_$LT$impl$u20$core..fmt..Display$u20$for$u20$u8$GT$3fmt17h47d2f4a6dbcaf6e5E>
   11c80: 60a2         	ld	ra, 0x8(sp)
   11c82: 6402         	ld	s0, 0x0(sp)
   11c84: 0141         	addi	sp, sp, 0x10
   11c86: 00002317     	auipc	t1, 0x2
   11c8a: 1a430067     	jr	0x1a4(t1) <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h75051e47a1d83752E>
   11c8e: 60a2         	ld	ra, 0x8(sp)
   11c90: 6402         	ld	s0, 0x0(sp)
   11c92: 0141         	addi	sp, sp, 0x10
   11c94: 00002317     	auipc	t1, 0x2
   11c98: 1f230067     	jr	0x1f2(t1) <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h42a55981b2e0dc80E>

0000000000011c9c <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17hc3e5d3410538c8eeE>:
   11c9c: 1141         	addi	sp, sp, -0x10
   11c9e: e406         	sd	ra, 0x8(sp)
   11ca0: e022         	sd	s0, 0x0(sp)
   11ca2: 0800         	addi	s0, sp, 0x10
   11ca4: 6108         	ld	a0, 0x0(a0)
   11ca6: 60a2         	ld	ra, 0x8(sp)
   11ca8: 6402         	ld	s0, 0x0(sp)
   11caa: 0141         	addi	sp, sp, 0x10
   11cac: 00001317     	auipc	t1, 0x1
   11cb0: b8230067     	jr	-0x47e(t1) <_ZN68_$LT$core..ptr..alignment..Alignment$u20$as$u20$core..fmt..Debug$GT$3fmt17hbd856da387d08de8E>

0000000000011cb4 <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h35f0a6453c985293E>:
   11cb4: 1141         	addi	sp, sp, -0x10
   11cb6: e406         	sd	ra, 0x8(sp)
   11cb8: e022         	sd	s0, 0x0(sp)
   11cba: 0800         	addi	s0, sp, 0x10
   11cbc: 6110         	ld	a2, 0x0(a0)
   11cbe: 6514         	ld	a3, 0x8(a0)
   11cc0: 872e         	mv	a4, a1
   11cc2: 8532         	mv	a0, a2
   11cc4: 85b6         	mv	a1, a3
   11cc6: 863a         	mv	a2, a4
   11cc8: 60a2         	ld	ra, 0x8(sp)
   11cca: 6402         	ld	s0, 0x0(sp)
   11ccc: 0141         	addi	sp, sp, 0x10
   11cce: 00002317     	auipc	t1, 0x2
   11cd2: afc30067     	jr	-0x504(t1) <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h6c34711bbfb2649bE>

0000000000011cd6 <_ZN4core3fmt5Write10write_char17h628937511f81a59eE>:
   11cd6: 1101         	addi	sp, sp, -0x20
   11cd8: ec06         	sd	ra, 0x18(sp)
   11cda: e822         	sd	s0, 0x10(sp)
   11cdc: 1000         	addi	s0, sp, 0x20
   11cde: 08000513     	li	a0, 0x80
   11ce2: fe042623     	sw	zero, -0x14(s0)
   11ce6: 00a5f663     	bgeu	a1, a0, 0x11cf2 <_ZN4core3fmt5Write10write_char17h628937511f81a59eE+0x1c>
   11cea: feb40623     	sb	a1, -0x14(s0)
   11cee: 4605         	li	a2, 0x1
   11cf0: a069         	j	0x11d7a <_ZN4core3fmt5Write10write_char17h628937511f81a59eE+0xa4>
   11cf2: 00b5d51b     	srliw	a0, a1, 0xb
   11cf6: ed19         	bnez	a0, 0x11d14 <_ZN4core3fmt5Write10write_char17h628937511f81a59eE+0x3e>
   11cf8: 0065d513     	srli	a0, a1, 0x6
   11cfc: 0c056513     	ori	a0, a0, 0xc0
   11d00: fea40623     	sb	a0, -0x14(s0)
   11d04: 03f5f513     	andi	a0, a1, 0x3f
   11d08: 08050513     	addi	a0, a0, 0x80
   11d0c: fea406a3     	sb	a0, -0x13(s0)
   11d10: 4609         	li	a2, 0x2
   11d12: a0a5         	j	0x11d7a <_ZN4core3fmt5Write10write_char17h628937511f81a59eE+0xa4>
   11d14: 0105d51b     	srliw	a0, a1, 0x10
   11d18: e515         	bnez	a0, 0x11d44 <_ZN4core3fmt5Write10write_char17h628937511f81a59eE+0x6e>
   11d1a: 00c5d513     	srli	a0, a1, 0xc
   11d1e: 0e056513     	ori	a0, a0, 0xe0
   11d22: fea40623     	sb	a0, -0x14(s0)
   11d26: 03459513     	slli	a0, a1, 0x34
   11d2a: 9169         	srli	a0, a0, 0x3a
   11d2c: 08050513     	addi	a0, a0, 0x80
   11d30: fea406a3     	sb	a0, -0x13(s0)
   11d34: 03f5f513     	andi	a0, a1, 0x3f
   11d38: 08050513     	addi	a0, a0, 0x80
   11d3c: fea40723     	sb	a0, -0x12(s0)
   11d40: 460d         	li	a2, 0x3
   11d42: a825         	j	0x11d7a <_ZN4core3fmt5Write10write_char17h628937511f81a59eE+0xa4>
   11d44: 0125d513     	srli	a0, a1, 0x12
   11d48: 0f056513     	ori	a0, a0, 0xf0
   11d4c: fea40623     	sb	a0, -0x14(s0)
   11d50: 02e59513     	slli	a0, a1, 0x2e
   11d54: 9169         	srli	a0, a0, 0x3a
   11d56: 08050513     	addi	a0, a0, 0x80
   11d5a: fea406a3     	sb	a0, -0x13(s0)
   11d5e: 03459513     	slli	a0, a1, 0x34
   11d62: 9169         	srli	a0, a0, 0x3a
   11d64: 08050513     	addi	a0, a0, 0x80
   11d68: fea40723     	sb	a0, -0x12(s0)
   11d6c: 03f5f513     	andi	a0, a1, 0x3f
   11d70: 08050513     	addi	a0, a0, 0x80
   11d74: fea407a3     	sb	a0, -0x11(s0)
   11d78: 4611         	li	a2, 0x4
   11d7a: 4505         	li	a0, 0x1
   11d7c: fec40593     	addi	a1, s0, -0x14
   11d80: 04000893     	li	a7, 0x40
   11d84: 00000073     	ecall
   11d88: 4501         	li	a0, 0x0
   11d8a: 60e2         	ld	ra, 0x18(sp)
   11d8c: 6442         	ld	s0, 0x10(sp)
   11d8e: 6105         	addi	sp, sp, 0x20
   11d90: 8082         	ret

0000000000011d92 <_ZN4core3fmt5Write9write_fmt17hf6328c7636f1cbe2E>:
   11d92: 1141         	addi	sp, sp, -0x10
   11d94: e406         	sd	ra, 0x8(sp)
   11d96: e022         	sd	s0, 0x0(sp)
   11d98: 0800         	addi	s0, sp, 0x10

0000000000011d9a <.Lpcrel_hi4>:
   11d9a: 00004617     	auipc	a2, 0x4
   11d9e: a8660613     	addi	a2, a2, -0x57a
   11da2: 86ae         	mv	a3, a1
   11da4: 85b2         	mv	a1, a2
   11da6: 8636         	mv	a2, a3
   11da8: 60a2         	ld	ra, 0x8(sp)
   11daa: 6402         	ld	s0, 0x0(sp)
   11dac: 0141         	addi	sp, sp, 0x10
   11dae: 00001317     	auipc	t1, 0x1
   11db2: 1fc30067     	jr	0x1fc(t1) <_ZN4core3fmt5write17h1b882e4f6891aa5dE>

0000000000011db6 <_ZN53_$LT$core..fmt..Error$u20$as$u20$core..fmt..Debug$GT$3fmt17h80422288f5eb0c29E>:
   11db6: 1141         	addi	sp, sp, -0x10
   11db8: e406         	sd	ra, 0x8(sp)
   11dba: e022         	sd	s0, 0x0(sp)
   11dbc: 0800         	addi	s0, sp, 0x10

0000000000011dbe <.Lpcrel_hi6>:
   11dbe: 00004517     	auipc	a0, 0x4
   11dc2: a5a50693     	addi	a3, a0, -0x5a6
   11dc6: 4615         	li	a2, 0x5
   11dc8: 852e         	mv	a0, a1
   11dca: 85b6         	mv	a1, a3
   11dcc: 60a2         	ld	ra, 0x8(sp)
   11dce: 6402         	ld	s0, 0x0(sp)
   11dd0: 0141         	addi	sp, sp, 0x10
   11dd2: 00001317     	auipc	t1, 0x1
   11dd6: 7cc30067     	jr	0x7cc(t1) <_ZN57_$LT$core..fmt..Formatter$u20$as$u20$core..fmt..Write$GT$9write_str17h41d63e4bf0c652c7E>

0000000000011dda <_ZN5alloc7raw_vec11finish_grow17h3b62e83d8ddf0a44E>:
   11dda: 7139         	addi	sp, sp, -0x40
   11ddc: fc06         	sd	ra, 0x38(sp)
   11dde: f822         	sd	s0, 0x30(sp)
   11de0: f426         	sd	s1, 0x28(sp)
   11de2: f04a         	sd	s2, 0x20(sp)
   11de4: ec4e         	sd	s3, 0x18(sp)
   11de6: e852         	sd	s4, 0x10(sp)
   11de8: e456         	sd	s5, 0x8(sp)
   11dea: 0080         	addi	s0, sp, 0x40
   11dec: 6614         	ld	a3, 0x8(a2)
   11dee: 892e         	mv	s2, a1
   11df0: 84aa         	mv	s1, a0
   11df2: cab1         	beqz	a3, 0x11e46 <.Lpcrel_hi10+0x22>
   11df4: 01063983     	ld	s3, 0x10(a2)
   11df8: 06098463     	beqz	s3, 0x11e60 <.Lpcrel_hi12+0xa>
   11dfc: 00063a03     	ld	s4, 0x0(a2)

0000000000011e00 <.Lpcrel_hi9>:
   11e00: 0000d517     	auipc	a0, 0xd
   11e04: 20050513     	addi	a0, a0, 0x200
   11e08: 45a1         	li	a1, 0x8
   11e0a: 864a         	mv	a2, s2
   11e0c: 00000097     	auipc	ra, 0x0
   11e10: 77c080e7     	jalr	0x77c(ra) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE>
   11e14: cd2d         	beqz	a0, 0x11e8e <.Lpcrel_hi8+0x1e>
   11e16: 8aaa         	mv	s5, a0
   11e18: 85d2         	mv	a1, s4
   11e1a: 864e         	mv	a2, s3
   11e1c: 00002097     	auipc	ra, 0x2
   11e20: 6fc080e7     	jalr	0x6fc(ra) <memcpy>

0000000000011e24 <.Lpcrel_hi10>:
   11e24: 0000d517     	auipc	a0, 0xd
   11e28: 1dc50513     	addi	a0, a0, 0x1dc
   11e2c: 4621         	li	a2, 0x8
   11e2e: 85d2         	mv	a1, s4
   11e30: 86ce         	mv	a3, s3
   11e32: 00000097     	auipc	ra, 0x0
   11e36: 7a4080e7     	jalr	0x7a4(ra) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7dealloc17h6485ec3a8003afbfE>
   11e3a: 8556         	mv	a0, s5
   11e3c: 001ab593     	seqz	a1, s5
   11e40: 040a8a63     	beqz	s5, 0x11e94 <.Lpcrel_hi8+0x24>
   11e44: a889         	j	0x11e96 <.Lpcrel_hi8+0x26>
   11e46: 04090363     	beqz	s2, 0x11e8c <.Lpcrel_hi8+0x1c>

0000000000011e4a <.Lpcrel_hi11>:
   11e4a: 0000d517     	auipc	a0, 0xd
   11e4e: 2df50513     	addi	a0, a0, 0x2df
   11e52: 00054003     	lbu	zero, 0x0(a0)

0000000000011e56 <.Lpcrel_hi12>:
   11e56: 0000d517     	auipc	a0, 0xd
   11e5a: 1aa50513     	addi	a0, a0, 0x1aa
   11e5e: a829         	j	0x11e78 <.Lpcrel_hi8+0x8>
   11e60: 04090863     	beqz	s2, 0x11eb0 <.Lpcrel_hi8+0x40>

0000000000011e64 <.Lpcrel_hi7>:
   11e64: 0000d517     	auipc	a0, 0xd
   11e68: 2c550513     	addi	a0, a0, 0x2c5
   11e6c: 00054003     	lbu	zero, 0x0(a0)

0000000000011e70 <.Lpcrel_hi8>:
   11e70: 0000d517     	auipc	a0, 0xd
   11e74: 19050513     	addi	a0, a0, 0x190
   11e78: 45a1         	li	a1, 0x8
   11e7a: 864a         	mv	a2, s2
   11e7c: 00000097     	auipc	ra, 0x0
   11e80: 70c080e7     	jalr	0x70c(ra) <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE>
   11e84: 00153593     	seqz	a1, a0
   11e88: c511         	beqz	a0, 0x11e94 <.Lpcrel_hi8+0x24>
   11e8a: a031         	j	0x11e96 <.Lpcrel_hi8+0x26>
   11e8c: 4521         	li	a0, 0x8
   11e8e: 00153593     	seqz	a1, a0
   11e92: e111         	bnez	a0, 0x11e96 <.Lpcrel_hi8+0x26>
   11e94: 4521         	li	a0, 0x8
   11e96: e488         	sd	a0, 0x8(s1)
   11e98: 0124b823     	sd	s2, 0x10(s1)
   11e9c: e08c         	sd	a1, 0x0(s1)
   11e9e: 70e2         	ld	ra, 0x38(sp)
   11ea0: 7442         	ld	s0, 0x30(sp)
   11ea2: 74a2         	ld	s1, 0x28(sp)
   11ea4: 7902         	ld	s2, 0x20(sp)
   11ea6: 69e2         	ld	s3, 0x18(sp)
   11ea8: 6a42         	ld	s4, 0x10(sp)
   11eaa: 6aa2         	ld	s5, 0x8(sp)
   11eac: 6121         	addi	sp, sp, 0x40
   11eae: 8082         	ret
   11eb0: 4521         	li	a0, 0x8
   11eb2: 00153593     	seqz	a1, a0
   11eb6: dd79         	beqz	a0, 0x11e94 <.Lpcrel_hi8+0x24>
   11eb8: bff9         	j	0x11e96 <.Lpcrel_hi8+0x26>

0000000000011eba <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE>:
   11eba: 711d         	addi	sp, sp, -0x60
   11ebc: ec86         	sd	ra, 0x58(sp)
   11ebe: e8a2         	sd	s0, 0x50(sp)
   11ec0: e4a6         	sd	s1, 0x48(sp)
   11ec2: e0ca         	sd	s2, 0x40(sp)
   11ec4: fc4e         	sd	s3, 0x38(sp)
   11ec6: 1080         	addi	s0, sp, 0x60
   11ec8: 84aa         	mv	s1, a0
   11eca: 6114         	ld	a3, 0x0(a0)
   11ecc: 00168613     	addi	a2, a3, 0x1
   11ed0: 00169513     	slli	a0, a3, 0x1
   11ed4: 892e         	mv	s2, a1
   11ed6: 02a67463     	bgeu	a2, a0, 0x11efe <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x44>
   11eda: 4591         	li	a1, 0x4
   11edc: 89aa         	mv	s3, a0
   11ede: 02a5f563     	bgeu	a1, a0, 0x11f08 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x4e>
   11ee2: 03c55593     	srli	a1, a0, 0x3c
   11ee6: 4501         	li	a0, 0x0
   11ee8: e58d         	bnez	a1, 0x11f12 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x58>
   11eea: 00499593     	slli	a1, s3, 0x4
   11eee: 5645         	li	a2, -0xf
   11ef0: 00165713     	srli	a4, a2, 0x1
   11ef4: 00b76f63     	bltu	a4, a1, 0x11f12 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x58>
   11ef8: e29d         	bnez	a3, 0x11f1e <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x64>
   11efa: 4501         	li	a0, 0x0
   11efc: a805         	j	0x11f2c <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x72>
   11efe: 8532         	mv	a0, a2
   11f00: 4591         	li	a1, 0x4
   11f02: 89aa         	mv	s3, a0
   11f04: fcc5efe3     	bltu	a1, a2, 0x11ee2 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x28>
   11f08: 4991         	li	s3, 0x4
   11f0a: 03c55593     	srli	a1, a0, 0x3c
   11f0e: 4501         	li	a0, 0x0
   11f10: dde9         	beqz	a1, 0x11eea <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0x30>
   11f12: 85b2         	mv	a1, a2
   11f14: 864a         	mv	a2, s2
   11f16: 00001097     	auipc	ra, 0x1
   11f1a: 866080e7     	jalr	-0x79a(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>
   11f1e: 6488         	ld	a0, 0x8(s1)
   11f20: 0692         	slli	a3, a3, 0x4
   11f22: fca43023     	sd	a0, -0x40(s0)
   11f26: fcd43823     	sd	a3, -0x30(s0)
   11f2a: 4521         	li	a0, 0x8
   11f2c: fca43423     	sd	a0, -0x38(s0)
   11f30: fa840513     	addi	a0, s0, -0x58
   11f34: fc040613     	addi	a2, s0, -0x40
   11f38: 00000097     	auipc	ra, 0x0
   11f3c: ea2080e7     	jalr	-0x15e(ra) <_ZN5alloc7raw_vec11finish_grow17h3b62e83d8ddf0a44E>
   11f40: fa843503     	ld	a0, -0x58(s0)
   11f44: ed09         	bnez	a0, 0x11f5e <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17h440667eeaba85e1eE+0xa4>
   11f46: fb043503     	ld	a0, -0x50(s0)
   11f4a: e488         	sd	a0, 0x8(s1)
   11f4c: 0134b023     	sd	s3, 0x0(s1)
   11f50: 60e6         	ld	ra, 0x58(sp)
   11f52: 6446         	ld	s0, 0x50(sp)
   11f54: 64a6         	ld	s1, 0x48(sp)
   11f56: 6906         	ld	s2, 0x40(sp)
   11f58: 79e2         	ld	s3, 0x38(sp)
   11f5a: 6125         	addi	sp, sp, 0x60
   11f5c: 8082         	ret
   11f5e: fb043503     	ld	a0, -0x50(s0)
   11f62: fb843603     	ld	a2, -0x48(s0)
   11f66: 85b2         	mv	a1, a2
   11f68: 864a         	mv	a2, s2
   11f6a: 00001097     	auipc	ra, 0x1
   11f6e: 812080e7     	jalr	-0x7ee(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

0000000000011f72 <_ZN76_$LT$core..panic..panic_info..PanicMessage$u20$as$u20$core..fmt..Display$GT$3fmt17hb3b6848845ec1d88E>:
   11f72: 7139         	addi	sp, sp, -0x40
   11f74: fc06         	sd	ra, 0x38(sp)
   11f76: f822         	sd	s0, 0x30(sp)
   11f78: 0080         	addi	s0, sp, 0x40
   11f7a: 6108         	ld	a0, 0x0(a0)
   11f7c: 6110         	ld	a2, 0x0(a0)
   11f7e: 6514         	ld	a3, 0x8(a0)
   11f80: 6918         	ld	a4, 0x10(a0)
   11f82: fcc43023     	sd	a2, -0x40(s0)
   11f86: fcd43423     	sd	a3, -0x38(s0)
   11f8a: fce43823     	sd	a4, -0x30(s0)
   11f8e: 6d10         	ld	a2, 0x18(a0)
   11f90: 7114         	ld	a3, 0x20(a0)
   11f92: 7518         	ld	a4, 0x28(a0)
   11f94: 7988         	ld	a0, 0x30(a1)
   11f96: 7d8c         	ld	a1, 0x38(a1)
   11f98: fcc43c23     	sd	a2, -0x28(s0)
   11f9c: fed43023     	sd	a3, -0x20(s0)
   11fa0: fee43423     	sd	a4, -0x18(s0)
   11fa4: fc040613     	addi	a2, s0, -0x40
   11fa8: 00001097     	auipc	ra, 0x1
   11fac: 002080e7     	jalr	0x2(ra) <_ZN4core3fmt5write17h1b882e4f6891aa5dE>
   11fb0: 70e2         	ld	ra, 0x38(sp)
   11fb2: 7442         	ld	s0, 0x30(sp)
   11fb4: 6121         	addi	sp, sp, 0x40
   11fb6: 8082         	ret

0000000000011fb8 <_ZN62_$LT$user_lib..console..Stdout$u20$as$u20$core..fmt..Write$GT$9write_str17h042a96915e2a032bE>:
   11fb8: 1141         	addi	sp, sp, -0x10
   11fba: e406         	sd	ra, 0x8(sp)
   11fbc: e022         	sd	s0, 0x0(sp)
   11fbe: 0800         	addi	s0, sp, 0x10
   11fc0: 4505         	li	a0, 0x1
   11fc2: 04000893     	li	a7, 0x40
   11fc6: 00000073     	ecall
   11fca: 4501         	li	a0, 0x0
   11fcc: 60a2         	ld	ra, 0x8(sp)
   11fce: 6402         	ld	s0, 0x0(sp)
   11fd0: 0141         	addi	sp, sp, 0x10
   11fd2: 8082         	ret

0000000000011fd4 <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>:
   11fd4: 1101         	addi	sp, sp, -0x20
   11fd6: ec06         	sd	ra, 0x18(sp)
   11fd8: e822         	sd	s0, 0x10(sp)
   11fda: 1000         	addi	s0, sp, 0x20
   11fdc: 862a         	mv	a2, a0

0000000000011fde <.Lpcrel_hi32>:
   11fde: 00004517     	auipc	a0, 0x4
   11fe2: 84250593     	addi	a1, a0, -0x7be
   11fe6: fef40513     	addi	a0, s0, -0x11
   11fea: 00001097     	auipc	ra, 0x1
   11fee: fc0080e7     	jalr	-0x40(ra) <_ZN4core3fmt5write17h1b882e4f6891aa5dE>
   11ff2: e509         	bnez	a0, 0x11ffc <.Lpcrel_hi33>
   11ff4: 60e2         	ld	ra, 0x18(sp)
   11ff6: 6442         	ld	s0, 0x10(sp)
   11ff8: 6105         	addi	sp, sp, 0x20
   11ffa: 8082         	ret

0000000000011ffc <.Lpcrel_hi33>:
   11ffc: 00004517     	auipc	a0, 0x4
   12000: 87450513     	addi	a0, a0, -0x78c

0000000000012004 <.Lpcrel_hi34>:
   12004: 00004597     	auipc	a1, 0x4
   12008: 84c58693     	addi	a3, a1, -0x7b4

000000000001200c <.Lpcrel_hi35>:
   1200c: 00004597     	auipc	a1, 0x4
   12010: 8a458713     	addi	a4, a1, -0x75c
   12014: 02b00593     	li	a1, 0x2b
   12018: fef40613     	addi	a2, s0, -0x11
   1201c: 00001097     	auipc	ra, 0x1
   12020: ad2080e7     	jalr	-0x52e(ra) <_ZN4core6result13unwrap_failed17h097de9be360fa68cE>

0000000000012024 <rust_begin_unwind>:
   12024: 7175         	addi	sp, sp, -0x90
   12026: e506         	sd	ra, 0x88(sp)
   12028: e122         	sd	s0, 0x80(sp)
   1202a: 0900         	addi	s0, sp, 0x90
   1202c: 610c         	ld	a1, 0x0(a0)
   1202e: 6508         	ld	a0, 0x8(a0)
   12030: f6b43823     	sd	a1, -0x90(s0)
   12034: 610c         	ld	a1, 0x0(a0)
   12036: 6510         	ld	a2, 0x8(a0)
   12038: 4908         	lw	a0, 0x10(a0)
   1203a: fcb43c23     	sd	a1, -0x28(s0)
   1203e: fec43023     	sd	a2, -0x20(s0)
   12042: fea42623     	sw	a0, -0x14(s0)
   12046: fd840513     	addi	a0, s0, -0x28
   1204a: faa43423     	sd	a0, -0x58(s0)

000000000001204e <.Lpcrel_hi36>:
   1204e: 00000517     	auipc	a0, 0x0
   12052: c6650513     	addi	a0, a0, -0x39a
   12056: faa43823     	sd	a0, -0x50(s0)
   1205a: fec40513     	addi	a0, s0, -0x14
   1205e: faa43c23     	sd	a0, -0x48(s0)

0000000000012062 <.Lpcrel_hi37>:
   12062: 00002517     	auipc	a0, 0x2
   12066: 09a50513     	addi	a0, a0, 0x9a
   1206a: fca43023     	sd	a0, -0x40(s0)
   1206e: f7040513     	addi	a0, s0, -0x90
   12072: fca43423     	sd	a0, -0x38(s0)

0000000000012076 <.Lpcrel_hi38>:
   12076: 00000517     	auipc	a0, 0x0
   1207a: efc50513     	addi	a0, a0, -0x104
   1207e: fca43823     	sd	a0, -0x30(s0)

0000000000012082 <.Lpcrel_hi39>:
   12082: 00004517     	auipc	a0, 0x4
   12086: 85650513     	addi	a0, a0, -0x7aa
   1208a: f6a43c23     	sd	a0, -0x88(s0)
   1208e: 4511         	li	a0, 0x4
   12090: f8a43023     	sd	a0, -0x80(s0)
   12094: f8043c23     	sd	zero, -0x68(s0)
   12098: fa840513     	addi	a0, s0, -0x58
   1209c: f8a43423     	sd	a0, -0x78(s0)
   120a0: 450d         	li	a0, 0x3
   120a2: f8a43823     	sd	a0, -0x70(s0)
   120a6: f7840513     	addi	a0, s0, -0x88
   120aa: 00000097     	auipc	ra, 0x0
   120ae: f2a080e7     	jalr	-0xd6(ra) <_ZN8user_lib7console5print17h4fec0023d4e16f3eE>
   120b2: 557d         	li	a0, -0x1
   120b4: 00000097     	auipc	ra, 0x0
   120b8: a20080e7     	jalr	-0x5e0(ra) <_ZN8user_lib4exit17h98d922c79b0ff27aE>

00000000000120bc <_ZN8user_lib7syscall8sys_exit17h3bae8de294916964E>:
   120bc: 1141         	addi	sp, sp, -0x10
   120be: e406         	sd	ra, 0x8(sp)
   120c0: e022         	sd	s0, 0x0(sp)
   120c2: 0800         	addi	s0, sp, 0x10
   120c4: 05d00893     	li	a7, 0x5d
   120c8: 4581         	li	a1, 0x0
   120ca: 4601         	li	a2, 0x0
   120cc: 00000073     	ecall

00000000000120d0 <.Lpcrel_hi40>:
   120d0: 00004517     	auipc	a0, 0x4
   120d4: 84850513     	addi	a0, a0, -0x7b8

00000000000120d8 <.Lpcrel_hi41>:
   120d8: 00004597     	auipc	a1, 0x4
   120dc: 86858613     	addi	a2, a1, -0x798
   120e0: 45dd         	li	a1, 0x17
   120e2: 00001097     	auipc	ra, 0x1
   120e6: 878080e7     	jalr	-0x788(ra) <_ZN4core9panicking5panic17h6952156bbcf3c8fdE>

00000000000120ea <_ZN22buddy_system_allocator4Heap4init17h873e3cda72e1c95fE>:
   120ea: 1141         	addi	sp, sp, -0x10
   120ec: e406         	sd	ra, 0x8(sp)
   120ee: e022         	sd	s0, 0x0(sp)
   120f0: 0800         	addi	s0, sp, 0x10
   120f2: 962e         	add	a2, a2, a1
   120f4: 059d         	addi	a1, a1, 0x7
   120f6: ff85f793     	andi	a5, a1, -0x8
   120fa: ff867f93     	andi	t6, a2, -0x8
   120fe: 10ffe963     	bltu	t6, a5, 0x12210 <.Lpcrel_hi39>
   12102: 4701         	li	a4, 0x0
   12104: 00878593     	addi	a1, a5, 0x8
   12108: 0ebfeb63     	bltu	t6, a1, 0x121fe <.Lpcrel_hi38+0xae>
   1210c: 4805         	li	a6, 0x1
   1210e: 48fd         	li	a7, 0x1f

0000000000012110 <.Lpcrel_hi37>:
   12110: 00003597     	auipc	a1, 0x3
   12114: 2485b283     	ld	t0, 0x248(a1)
   12118: 555555b7     	lui	a1, 0x55555
   1211c: 5555831b     	addiw	t1, a1, 0x555
   12120: 02031593     	slli	a1, t1, 0x20
   12124: 932e         	add	t1, t1, a1
   12126: 333335b7     	lui	a1, 0x33333
   1212a: 33358f1b     	addiw	t5, a1, 0x333
   1212e: 020f1593     	slli	a1, t5, 0x20
   12132: 9f2e         	add	t5, t5, a1
   12134: 0f0f15b7     	lui	a1, 0xf0f1
   12138: f0f5839b     	addiw	t2, a1, -0xf1
   1213c: 02039593     	slli	a1, t2, 0x20
   12140: 93ae         	add	t2, t2, a1
   12142: 010105b7     	lui	a1, 0x1010
   12146: 10158e1b     	addiw	t3, a1, 0x101
   1214a: 020e1593     	slli	a1, t3, 0x20
   1214e: 9e2e         	add	t3, t3, a1

0000000000012150 <.Lpcrel_hi38>:
   12150: 00004597     	auipc	a1, 0x4
   12154: 94858e93     	addi	t4, a1, -0x6b8
   12158: 40ff85b3     	sub	a1, t6, a5
   1215c: c9a9         	beqz	a1, 0x121ae <.Lpcrel_hi38+0x5e>
   1215e: 0015d693     	srli	a3, a1, 0x1
   12162: 8dd5         	or	a1, a1, a3
   12164: 0025d693     	srli	a3, a1, 0x2
   12168: 8dd5         	or	a1, a1, a3
   1216a: 0045d693     	srli	a3, a1, 0x4
   1216e: 8dd5         	or	a1, a1, a3
   12170: 0085d693     	srli	a3, a1, 0x8
   12174: 8dd5         	or	a1, a1, a3
   12176: 0105d693     	srli	a3, a1, 0x10
   1217a: 8dd5         	or	a1, a1, a3
   1217c: 0205d693     	srli	a3, a1, 0x20
   12180: 8dd5         	or	a1, a1, a3
   12182: fff5c593     	not	a1, a1
   12186: 0015d693     	srli	a3, a1, 0x1
   1218a: 0066f6b3     	and	a3, a3, t1
   1218e: 8d95         	sub	a1, a1, a3
   12190: 01e5f6b3     	and	a3, a1, t5
   12194: 8189         	srli	a1, a1, 0x2
   12196: 01e5f5b3     	and	a1, a1, t5
   1219a: 95b6         	add	a1, a1, a3
   1219c: 0045d693     	srli	a3, a1, 0x4
   121a0: 95b6         	add	a1, a1, a3
   121a2: 0075f5b3     	and	a1, a1, t2
   121a6: 03c586b3     	mul	a3, a1, t3
   121aa: 92e1         	srli	a3, a3, 0x38
   121ac: a019         	j	0x121b2 <.Lpcrel_hi38+0x62>
   121ae: 04000693     	li	a3, 0x40
   121b2: 40f005b3     	neg	a1, a5
   121b6: 8dfd         	and	a1, a1, a5
   121b8: fff6c693     	not	a3, a3
   121bc: 00d816b3     	sll	a3, a6, a3
   121c0: 00d5e363     	bltu	a1, a3, 0x121c6 <.Lpcrel_hi38+0x76>
   121c4: 85b6         	mv	a1, a3
   121c6: cd89         	beqz	a1, 0x121e0 <.Lpcrel_hi38+0x90>
   121c8: 40b006b3     	neg	a3, a1
   121cc: 8eed         	and	a3, a3, a1
   121ce: 025686b3     	mul	a3, a3, t0
   121d2: 92e9         	srli	a3, a3, 0x3a
   121d4: 96f6         	add	a3, a3, t4
   121d6: 0006c683     	lbu	a3, 0x0(a3)
   121da: 00d8f763     	bgeu	a7, a3, 0x121e8 <.Lpcrel_hi38+0x98>
   121de: a0b1         	j	0x1222a <.Lpcrel_hi41>
   121e0: 04000693     	li	a3, 0x40
   121e4: 04d8e363     	bltu	a7, a3, 0x1222a <.Lpcrel_hi41>
   121e8: 068e         	slli	a3, a3, 0x3
   121ea: 96aa         	add	a3, a3, a0
   121ec: 6290         	ld	a2, 0x0(a3)
   121ee: e390         	sd	a2, 0x0(a5)
   121f0: e29c         	sd	a5, 0x0(a3)
   121f2: 97ae         	add	a5, a5, a1
   121f4: 00878613     	addi	a2, a5, 0x8
   121f8: 972e         	add	a4, a4, a1
   121fa: f4cfffe3     	bgeu	t6, a2, 0x12158 <.Lpcrel_hi38+0x8>
   121fe: 11053583     	ld	a1, 0x110(a0)
   12202: 95ba         	add	a1, a1, a4
   12204: 10b53823     	sd	a1, 0x110(a0)
   12208: 60a2         	ld	ra, 0x8(sp)
   1220a: 6402         	ld	s0, 0x0(sp)
   1220c: 0141         	addi	sp, sp, 0x10
   1220e: 8082         	ret

0000000000012210 <.Lpcrel_hi39>:
   12210: 00004517     	auipc	a0, 0x4
   12214: 94850513     	addi	a0, a0, -0x6b8

0000000000012218 <.Lpcrel_hi40>:
   12218: 00004597     	auipc	a1, 0x4
   1221c: 9c858613     	addi	a2, a1, -0x638
   12220: 45f9         	li	a1, 0x1e
   12222: 00000097     	auipc	ra, 0x0
   12226: 738080e7     	jalr	0x738(ra) <_ZN4core9panicking5panic17h6952156bbcf3c8fdE>

000000000001222a <.Lpcrel_hi41>:
   1222a: 00004517     	auipc	a0, 0x4
   1222e: 9ce50613     	addi	a2, a0, -0x632
   12232: 02000593     	li	a1, 0x20
   12236: 8536         	mv	a0, a3
   12238: 00000097     	auipc	ra, 0x0
   1223c: 75c080e7     	jalr	0x75c(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

0000000000012240 <_ZN22buddy_system_allocator4Heap5alloc17h9fe7e10c8968050dE>:
   12240: 1141         	addi	sp, sp, -0x10
   12242: e406         	sd	ra, 0x8(sp)
   12244: e022         	sd	s0, 0x0(sp)
   12246: 0800         	addi	s0, sp, 0x10
   12248: 4685         	li	a3, 0x1
   1224a: 0cc6e563     	bltu	a3, a2, 0x12314 <.Lpcrel_hi43+0xa6>
   1224e: 14b6fc63     	bgeu	a3, a1, 0x123a6 <.Lpcrel_hi43+0x138>
   12252: 46a1         	li	a3, 0x8
   12254: 00b6e363     	bltu	a3, a1, 0x1225a <.Lpcrel_hi42>
   12258: 45a1         	li	a1, 0x8

000000000001225a <.Lpcrel_hi42>:
   1225a: 00003697     	auipc	a3, 0x3
   1225e: 0fe6b683     	ld	a3, 0xfe(a3)
   12262: 40b00733     	neg	a4, a1
   12266: 8f6d         	and	a4, a4, a1
   12268: 02d706b3     	mul	a3, a4, a3
   1226c: 92e9         	srli	a3, a3, 0x3a

000000000001226e <.Lpcrel_hi43>:
   1226e: 00004717     	auipc	a4, 0x4
   12272: 86a70713     	addi	a4, a4, -0x796
   12276: 96ba         	add	a3, a3, a4
   12278: 0006c883     	lbu	a7, 0x0(a3)
   1227c: 02000693     	li	a3, 0x20
   12280: 82c6         	mv	t0, a7
   12282: 0116e463     	bltu	a3, a7, 0x1228a <.Lpcrel_hi43+0x1c>
   12286: 02000293     	li	t0, 0x20
   1228a: 00389813     	slli	a6, a7, 0x3
   1228e: 982a         	add	a6, a6, a0
   12290: ff080793     	addi	a5, a6, -0x10
   12294: 8746         	mv	a4, a7
   12296: 06e28963     	beq	t0, a4, 0x12308 <.Lpcrel_hi43+0x9a>
   1229a: 0107be03     	ld	t3, 0x10(a5)
   1229e: 0705         	addi	a4, a4, 0x1
   122a0: 07a1         	addi	a5, a5, 0x8
   122a2: fe0e0ae3     	beqz	t3, 0x12296 <.Lpcrel_hi43+0x28>
   122a6: fff70693     	addi	a3, a4, -0x1
   122aa: 02d8fb63     	bgeu	a7, a3, 0x122e0 <.Lpcrel_hi43+0x72>
   122ae: 000e3383     	ld	t2, 0x0(t3)
   122b2: 00188293     	addi	t0, a7, 0x1
   122b6: 4305         	li	t1, 0x1
   122b8: fff70693     	addi	a3, a4, -0x1
   122bc: 0077b423     	sd	t2, 0x8(a5)
   122c0: 0007be83     	ld	t4, 0x0(a5)
   122c4: 1779         	addi	a4, a4, -0x2
   122c6: 00e313b3     	sll	t2, t1, a4
   122ca: 93f2         	add	t2, t2, t3
   122cc: 01d3b023     	sd	t4, 0x0(t2)
   122d0: 007e3023     	sd	t2, 0x0(t3)
   122d4: 01c7b023     	sd	t3, 0x0(a5)
   122d8: 17e1         	addi	a5, a5, -0x8
   122da: 8736         	mv	a4, a3
   122dc: fcd2eee3     	bltu	t0, a3, 0x122b8 <.Lpcrel_hi43+0x4a>
   122e0: 46fd         	li	a3, 0x1f
   122e2: 0d16e763     	bltu	a3, a7, 0x123b0 <.Lpcrel_hi44>
   122e6: 00083683     	ld	a3, 0x0(a6)
   122ea: cef1         	beqz	a3, 0x123c6 <.Lpcrel_hi45>
   122ec: 6298         	ld	a4, 0x0(a3)
   122ee: 00e83023     	sd	a4, 0x0(a6)
   122f2: 10053703     	ld	a4, 0x100(a0)
   122f6: 10853783     	ld	a5, 0x108(a0)
   122fa: 963a         	add	a2, a2, a4
   122fc: 10c53023     	sd	a2, 0x100(a0)
   12300: 95be         	add	a1, a1, a5
   12302: 10b53423     	sd	a1, 0x108(a0)
   12306: a011         	j	0x1230a <.Lpcrel_hi43+0x9c>
   12308: 4681         	li	a3, 0x0
   1230a: 8536         	mv	a0, a3
   1230c: 60a2         	ld	ra, 0x8(sp)
   1230e: 6402         	ld	s0, 0x0(sp)
   12310: 0141         	addi	sp, sp, 0x10
   12312: 8082         	ret
   12314: fff60693     	addi	a3, a2, -0x1
   12318: 0016d713     	srli	a4, a3, 0x1
   1231c: 8ed9         	or	a3, a3, a4
   1231e: 0026d713     	srli	a4, a3, 0x2
   12322: 8ed9         	or	a3, a3, a4
   12324: 0046d713     	srli	a4, a3, 0x4
   12328: 8ed9         	or	a3, a3, a4
   1232a: 0086d713     	srli	a4, a3, 0x8
   1232e: 8ed9         	or	a3, a3, a4
   12330: 0106d713     	srli	a4, a3, 0x10
   12334: 8ed9         	or	a3, a3, a4
   12336: 0206d713     	srli	a4, a3, 0x20
   1233a: 8ed9         	or	a3, a3, a4
   1233c: fff6c693     	not	a3, a3
   12340: 0016d813     	srli	a6, a3, 0x1
   12344: 555557b7     	lui	a5, 0x55555
   12348: 5557871b     	addiw	a4, a5, 0x555
   1234c: 02071793     	slli	a5, a4, 0x20
   12350: 973e         	add	a4, a4, a5
   12352: 00e87733     	and	a4, a6, a4
   12356: 8e99         	sub	a3, a3, a4
   12358: 33333737     	lui	a4, 0x33333
   1235c: 3337071b     	addiw	a4, a4, 0x333
   12360: 02071793     	slli	a5, a4, 0x20
   12364: 973e         	add	a4, a4, a5
   12366: 00e6f7b3     	and	a5, a3, a4
   1236a: 8289         	srli	a3, a3, 0x2
   1236c: 8ef9         	and	a3, a3, a4
   1236e: 96be         	add	a3, a3, a5
   12370: 0046d713     	srli	a4, a3, 0x4
   12374: 96ba         	add	a3, a3, a4
   12376: 0f0f1737     	lui	a4, 0xf0f1
   1237a: f0f7071b     	addiw	a4, a4, -0xf1
   1237e: 02071793     	slli	a5, a4, 0x20
   12382: 973e         	add	a4, a4, a5
   12384: 8ef9         	and	a3, a3, a4
   12386: 01010737     	lui	a4, 0x1010
   1238a: 1017071b     	addiw	a4, a4, 0x101
   1238e: 02071793     	slli	a5, a4, 0x20
   12392: 973e         	add	a4, a4, a5
   12394: 02e686b3     	mul	a3, a3, a4
   12398: 92e1         	srli	a3, a3, 0x38
   1239a: 577d         	li	a4, -0x1
   1239c: 00d756b3     	srl	a3, a4, a3
   123a0: 0685         	addi	a3, a3, 0x1
   123a2: eab6e8e3     	bltu	a3, a1, 0x12252 <_ZN22buddy_system_allocator4Heap5alloc17h9fe7e10c8968050dE+0x12>
   123a6: 85b6         	mv	a1, a3
   123a8: 46a1         	li	a3, 0x8
   123aa: eab6f7e3     	bgeu	a3, a1, 0x12258 <_ZN22buddy_system_allocator4Heap5alloc17h9fe7e10c8968050dE+0x18>
   123ae: b575         	j	0x1225a <.Lpcrel_hi42>

00000000000123b0 <.Lpcrel_hi44>:
   123b0: 00004517     	auipc	a0, 0x4
   123b4: 88850613     	addi	a2, a0, -0x778
   123b8: 02000593     	li	a1, 0x20
   123bc: 8546         	mv	a0, a7
   123be: 00000097     	auipc	ra, 0x0
   123c2: 5d6080e7     	jalr	0x5d6(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

00000000000123c6 <.Lpcrel_hi45>:
   123c6: 00004517     	auipc	a0, 0x4
   123ca: 84a50513     	addi	a0, a0, -0x7b6

00000000000123ce <.Lpcrel_hi46>:
   123ce: 00004597     	auipc	a1, 0x4
   123d2: 88258613     	addi	a2, a1, -0x77e
   123d6: 02800593     	li	a1, 0x28
   123da: 00000097     	auipc	ra, 0x0
   123de: 50a080e7     	jalr	0x50a(ra) <_ZN4core6option13expect_failed17he7ca34d564694790E>

00000000000123e2 <_ZN22buddy_system_allocator4Heap7dealloc17hd3c7b3c787c82001E>:
   123e2: 1141         	addi	sp, sp, -0x10
   123e4: e406         	sd	ra, 0x8(sp)
   123e6: e022         	sd	s0, 0x0(sp)
   123e8: 0800         	addi	s0, sp, 0x10
   123ea: 4705         	li	a4, 0x1
   123ec: 0cd76563     	bltu	a4, a3, 0x124b6 <.Lpcrel_hi48+0xa6>
   123f0: 16c77063     	bgeu	a4, a2, 0x12550 <.Lpcrel_hi48+0x140>
   123f4: 4721         	li	a4, 0x8
   123f6: 00c76363     	bltu	a4, a2, 0x123fc <.Lpcrel_hi47>
   123fa: 4621         	li	a2, 0x8

00000000000123fc <.Lpcrel_hi47>:
   123fc: 00003717     	auipc	a4, 0x3
   12400: f5c73703     	ld	a4, -0xa4(a4)
   12404: 40c007b3     	neg	a5, a2
   12408: 8ff1         	and	a5, a5, a2
   1240a: 02e78733     	mul	a4, a5, a4
   1240e: 9369         	srli	a4, a4, 0x3a

0000000000012410 <.Lpcrel_hi48>:
   12410: 00003797     	auipc	a5, 0x3
   12414: 70878793     	addi	a5, a5, 0x708
   12418: 973e         	add	a4, a4, a5
   1241a: 00074383     	lbu	t2, 0x0(a4)
   1241e: 48fd         	li	a7, 0x1f
   12420: 1478e963     	bltu	a7, t2, 0x12572 <.Lpcrel_hi49>
   12424: 00339713     	slli	a4, t2, 0x3
   12428: 972a         	add	a4, a4, a0
   1242a: 631c         	ld	a5, 0x0(a4)
   1242c: e19c         	sd	a5, 0x0(a1)
   1242e: e30c         	sd	a1, 0x0(a4)
   12430: 4705         	li	a4, 0x1
   12432: 00771333     	sll	t1, a4, t2
   12436: 00b34eb3     	xor	t4, t1, a1
   1243a: 4809         	li	a6, 0x2
   1243c: 82ae         	mv	t0, a1
   1243e: 00339e13     	slli	t3, t2, 0x3
   12442: 9e2a         	add	t3, t3, a0
   12444: 8772         	mv	a4, t3
   12446: 00be8863     	beq	t4, a1, 0x12456 <.Lpcrel_hi48+0x46>
   1244a: cba1         	beqz	a5, 0x1249a <.Lpcrel_hi48+0x8a>
   1244c: 872e         	mv	a4, a1
   1244e: 85be         	mv	a1, a5
   12450: 639c         	ld	a5, 0x0(a5)
   12452: febe9ce3     	bne	t4, a1, 0x1244a <.Lpcrel_hi48+0x3a>
   12456: e31c         	sd	a5, 0x0(a4)
   12458: 000e3583     	ld	a1, 0x0(t3)
   1245c: c581         	beqz	a1, 0x12464 <.Lpcrel_hi48+0x54>
   1245e: 618c         	ld	a1, 0x0(a1)
   12460: 00be3023     	sd	a1, 0x0(t3)
   12464: 0f138b63     	beq	t2, a7, 0x1255a <.Lpcrel_hi50>
   12468: 00138e13     	addi	t3, t2, 0x1
   1246c: 003e1593     	slli	a1, t3, 0x3
   12470: 00b50733     	add	a4, a0, a1
   12474: 631c         	ld	a5, 0x0(a4)
   12476: fff34593     	not	a1, t1
   1247a: 00781333     	sll	t1, a6, t2
   1247e: 00b2f5b3     	and	a1, t0, a1
   12482: e19c         	sd	a5, 0x0(a1)
   12484: e30c         	sd	a1, 0x0(a4)
   12486: 00b34eb3     	xor	t4, t1, a1
   1248a: 82ae         	mv	t0, a1
   1248c: 83f2         	mv	t2, t3
   1248e: 0e0e         	slli	t3, t3, 0x3
   12490: 9e2a         	add	t3, t3, a0
   12492: 8772         	mv	a4, t3
   12494: fabe9be3     	bne	t4, a1, 0x1244a <.Lpcrel_hi48+0x3a>
   12498: bf7d         	j	0x12456 <.Lpcrel_hi48+0x46>
   1249a: 10053583     	ld	a1, 0x100(a0)
   1249e: 10853703     	ld	a4, 0x108(a0)
   124a2: 8d95         	sub	a1, a1, a3
   124a4: 10b53023     	sd	a1, 0x100(a0)
   124a8: 8f11         	sub	a4, a4, a2
   124aa: 10e53423     	sd	a4, 0x108(a0)
   124ae: 60a2         	ld	ra, 0x8(sp)
   124b0: 6402         	ld	s0, 0x0(sp)
   124b2: 0141         	addi	sp, sp, 0x10
   124b4: 8082         	ret
   124b6: fff68713     	addi	a4, a3, -0x1
   124ba: 00175793     	srli	a5, a4, 0x1
   124be: 8f5d         	or	a4, a4, a5
   124c0: 00275793     	srli	a5, a4, 0x2
   124c4: 8f5d         	or	a4, a4, a5
   124c6: 00475793     	srli	a5, a4, 0x4
   124ca: 8f5d         	or	a4, a4, a5
   124cc: 00875793     	srli	a5, a4, 0x8
   124d0: 8f5d         	or	a4, a4, a5
   124d2: 01075793     	srli	a5, a4, 0x10
   124d6: 8f5d         	or	a4, a4, a5
   124d8: 02075793     	srli	a5, a4, 0x20
   124dc: 8f5d         	or	a4, a4, a5
   124de: fff74893     	not	a7, a4
   124e2: 0018d813     	srli	a6, a7, 0x1
   124e6: 555557b7     	lui	a5, 0x55555
   124ea: 5557871b     	addiw	a4, a5, 0x555
   124ee: 02071793     	slli	a5, a4, 0x20
   124f2: 973e         	add	a4, a4, a5
   124f4: 00e87733     	and	a4, a6, a4
   124f8: 40e88833     	sub	a6, a7, a4
   124fc: 333337b7     	lui	a5, 0x33333
   12500: 3337871b     	addiw	a4, a5, 0x333
   12504: 02071793     	slli	a5, a4, 0x20
   12508: 973e         	add	a4, a4, a5
   1250a: 00e878b3     	and	a7, a6, a4
   1250e: 00285793     	srli	a5, a6, 0x2
   12512: 8f7d         	and	a4, a4, a5
   12514: 9746         	add	a4, a4, a7
   12516: 00475793     	srli	a5, a4, 0x4
   1251a: 00f70833     	add	a6, a4, a5
   1251e: 0f0f17b7     	lui	a5, 0xf0f1
   12522: f0f7871b     	addiw	a4, a5, -0xf1
   12526: 02071793     	slli	a5, a4, 0x20
   1252a: 973e         	add	a4, a4, a5
   1252c: 00e87833     	and	a6, a6, a4
   12530: 010107b7     	lui	a5, 0x1010
   12534: 1017871b     	addiw	a4, a5, 0x101
   12538: 02071793     	slli	a5, a4, 0x20
   1253c: 973e         	add	a4, a4, a5
   1253e: 02e80733     	mul	a4, a6, a4
   12542: 9361         	srli	a4, a4, 0x38
   12544: 57fd         	li	a5, -0x1
   12546: 00e7d733     	srl	a4, a5, a4
   1254a: 0705         	addi	a4, a4, 0x1
   1254c: eac764e3     	bltu	a4, a2, 0x123f4 <_ZN22buddy_system_allocator4Heap7dealloc17hd3c7b3c787c82001E+0x12>
   12550: 863a         	mv	a2, a4
   12552: 4721         	li	a4, 0x8
   12554: eac773e3     	bgeu	a4, a2, 0x123fa <_ZN22buddy_system_allocator4Heap7dealloc17hd3c7b3c787c82001E+0x18>
   12558: b555         	j	0x123fc <.Lpcrel_hi47>

000000000001255a <.Lpcrel_hi50>:
   1255a: 00003517     	auipc	a0, 0x3
   1255e: 72650613     	addi	a2, a0, 0x726
   12562: 02000513     	li	a0, 0x20
   12566: 02000593     	li	a1, 0x20
   1256a: 00000097     	auipc	ra, 0x0
   1256e: 42a080e7     	jalr	0x42a(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

0000000000012572 <.Lpcrel_hi49>:
   12572: 00003517     	auipc	a0, 0x3
   12576: 6f650613     	addi	a2, a0, 0x6f6
   1257a: 02000593     	li	a1, 0x20
   1257e: 851e         	mv	a0, t2
   12580: 00000097     	auipc	ra, 0x0
   12584: 414080e7     	jalr	0x414(ra) <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>

0000000000012588 <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE>:
   12588: 1101         	addi	sp, sp, -0x20
   1258a: ec06         	sd	ra, 0x18(sp)
   1258c: e822         	sd	s0, 0x10(sp)
   1258e: e426         	sd	s1, 0x8(sp)
   12590: e04a         	sd	s2, 0x0(sp)
   12592: 1000         	addi	s0, sp, 0x20
   12594: 84aa         	mv	s1, a0
   12596: 4505         	li	a0, 0x1
   12598: 00a4b92f     	amoadd.d	s2, a0, (s1)
   1259c: 6488         	ld	a0, 0x8(s1)
   1259e: 0230000f     	fence	r, rw
   125a2: 01250963     	beq	a0, s2, 0x125b4 <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE+0x2c>
   125a6: 0100000f     	fence	w, 0
   125aa: 6488         	ld	a0, 0x8(s1)
   125ac: 0230000f     	fence	r, rw
   125b0: ff251be3     	bne	a0, s2, 0x125a6 <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$5alloc17h076e06a768aed75fE+0x1e>
   125b4: 01048513     	addi	a0, s1, 0x10
   125b8: 00000097     	auipc	ra, 0x0
   125bc: c88080e7     	jalr	-0x378(ra) <_ZN22buddy_system_allocator4Heap5alloc17h9fe7e10c8968050dE>
   125c0: 0905         	addi	s2, s2, 0x1
   125c2: 0310000f     	fence	rw, w
   125c6: 0124b423     	sd	s2, 0x8(s1)
   125ca: 60e2         	ld	ra, 0x18(sp)
   125cc: 6442         	ld	s0, 0x10(sp)
   125ce: 64a2         	ld	s1, 0x8(sp)
   125d0: 6902         	ld	s2, 0x0(sp)
   125d2: 6105         	addi	sp, sp, 0x20
   125d4: 8082         	ret

00000000000125d6 <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7dealloc17h6485ec3a8003afbfE>:
   125d6: 1101         	addi	sp, sp, -0x20
   125d8: ec06         	sd	ra, 0x18(sp)
   125da: e822         	sd	s0, 0x10(sp)
   125dc: e426         	sd	s1, 0x8(sp)
   125de: e04a         	sd	s2, 0x0(sp)
   125e0: 1000         	addi	s0, sp, 0x20
   125e2: 84aa         	mv	s1, a0
   125e4: 4505         	li	a0, 0x1
   125e6: 00a4b92f     	amoadd.d	s2, a0, (s1)
   125ea: 6488         	ld	a0, 0x8(s1)
   125ec: 0230000f     	fence	r, rw
   125f0: 01250963     	beq	a0, s2, 0x12602 <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7dealloc17h6485ec3a8003afbfE+0x2c>
   125f4: 0100000f     	fence	w, 0
   125f8: 6488         	ld	a0, 0x8(s1)
   125fa: 0230000f     	fence	r, rw
   125fe: ff251be3     	bne	a0, s2, 0x125f4 <_ZN87_$LT$buddy_system_allocator..LockedHeap$u20$as$u20$core..alloc..global..GlobalAlloc$GT$7dealloc17h6485ec3a8003afbfE+0x1e>
   12602: 01048513     	addi	a0, s1, 0x10
   12606: 00000097     	auipc	ra, 0x0
   1260a: ddc080e7     	jalr	-0x224(ra) <_ZN22buddy_system_allocator4Heap7dealloc17hd3c7b3c787c82001E>
   1260e: 0905         	addi	s2, s2, 0x1
   12610: 0310000f     	fence	rw, w
   12614: 0124b423     	sd	s2, 0x8(s1)
   12618: 60e2         	ld	ra, 0x18(sp)
   1261a: 6442         	ld	s0, 0x10(sp)
   1261c: 64a2         	ld	s1, 0x8(sp)
   1261e: 6902         	ld	s2, 0x0(sp)
   12620: 6105         	addi	sp, sp, 0x20
   12622: 8082         	ret

0000000000012624 <_ZN5alloc7raw_vec17capacity_overflow17hf6eb47088a69ee65E>:
   12624: 7139         	addi	sp, sp, -0x40
   12626: fc06         	sd	ra, 0x38(sp)
   12628: f822         	sd	s0, 0x30(sp)
   1262a: 0080         	addi	s0, sp, 0x40
   1262c: 85aa         	mv	a1, a0

000000000001262e <.Lpcrel_hi9>:
   1262e: 00003517     	auipc	a0, 0x3
   12632: 68250513     	addi	a0, a0, 0x682
   12636: fca43023     	sd	a0, -0x40(s0)
   1263a: 4505         	li	a0, 0x1
   1263c: fca43423     	sd	a0, -0x38(s0)
   12640: fe043023     	sd	zero, -0x20(s0)
   12644: 4521         	li	a0, 0x8
   12646: fca43823     	sd	a0, -0x30(s0)
   1264a: fc043c23     	sd	zero, -0x28(s0)
   1264e: fc040513     	addi	a0, s0, -0x40
   12652: 00000097     	auipc	ra, 0x0
   12656: 2e6080e7     	jalr	0x2e6(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

000000000001265a <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE>:
   1265a: 711d         	addi	sp, sp, -0x60
   1265c: ec86         	sd	ra, 0x58(sp)
   1265e: e8a2         	sd	s0, 0x50(sp)
   12660: e4a6         	sd	s1, 0x48(sp)
   12662: e0ca         	sd	s2, 0x40(sp)
   12664: fc4e         	sd	s3, 0x38(sp)
   12666: 1080         	addi	s0, sp, 0x60
   12668: 89aa         	mv	s3, a0
   1266a: 6108         	ld	a0, 0x0(a0)
   1266c: 00150613     	addi	a2, a0, 0x1
   12670: 00151493     	slli	s1, a0, 0x1
   12674: 892e         	mv	s2, a1
   12676: 04967b63     	bgeu	a2, s1, 0x126cc <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x72>
   1267a: 45a1         	li	a1, 0x8
   1267c: 0495fc63     	bgeu	a1, s1, 0x126d4 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x7a>
   12680: 0404cd63     	bltz	s1, 0x126da <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x80>
   12684: c901         	beqz	a0, 0x12694 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x3a>
   12686: 0089b583     	ld	a1, 0x8(s3)
   1268a: fcb43023     	sd	a1, -0x40(s0)
   1268e: fca43823     	sd	a0, -0x30(s0)
   12692: 4505         	li	a0, 0x1
   12694: fca43423     	sd	a0, -0x38(s0)
   12698: fa840513     	addi	a0, s0, -0x58
   1269c: 4585         	li	a1, 0x1
   1269e: fc040693     	addi	a3, s0, -0x40
   126a2: 8626         	mv	a2, s1
   126a4: 00000097     	auipc	ra, 0x0
   126a8: 054080e7     	jalr	0x54(ra) <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E>
   126ac: fa843503     	ld	a0, -0x58(s0)
   126b0: e91d         	bnez	a0, 0x126e6 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x8c>
   126b2: fb043503     	ld	a0, -0x50(s0)
   126b6: 00a9b423     	sd	a0, 0x8(s3)
   126ba: 0099b023     	sd	s1, 0x0(s3)
   126be: 60e6         	ld	ra, 0x58(sp)
   126c0: 6446         	ld	s0, 0x50(sp)
   126c2: 64a6         	ld	s1, 0x48(sp)
   126c4: 6906         	ld	s2, 0x40(sp)
   126c6: 79e2         	ld	s3, 0x38(sp)
   126c8: 6125         	addi	sp, sp, 0x60
   126ca: 8082         	ret
   126cc: 84b2         	mv	s1, a2
   126ce: 45a1         	li	a1, 0x8
   126d0: fac5e8e3     	bltu	a1, a2, 0x12680 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x26>
   126d4: 44a1         	li	s1, 0x8
   126d6: fa04d7e3     	bgez	s1, 0x12684 <_ZN5alloc7raw_vec19RawVec$LT$T$C$A$GT$8grow_one17hbab27e50577efebbE+0x2a>
   126da: 4501         	li	a0, 0x0
   126dc: 864a         	mv	a2, s2
   126de: 00000097     	auipc	ra, 0x0
   126e2: 09e080e7     	jalr	0x9e(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>
   126e6: fb043503     	ld	a0, -0x50(s0)
   126ea: fb843583     	ld	a1, -0x48(s0)
   126ee: 864a         	mv	a2, s2
   126f0: 00000097     	auipc	ra, 0x0
   126f4: 08c080e7     	jalr	0x8c(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

00000000000126f8 <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E>:
   126f8: 7179         	addi	sp, sp, -0x30
   126fa: f406         	sd	ra, 0x28(sp)
   126fc: f022         	sd	s0, 0x20(sp)
   126fe: ec26         	sd	s1, 0x18(sp)
   12700: e84a         	sd	s2, 0x10(sp)
   12702: e44e         	sd	s3, 0x8(sp)
   12704: 1800         	addi	s0, sp, 0x30
   12706: 6698         	ld	a4, 0x8(a3)
   12708: 8932         	mv	s2, a2
   1270a: 89ae         	mv	s3, a1
   1270c: 84aa         	mv	s1, a0
   1270e: cb15         	beqz	a4, 0x12742 <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E+0x4a>
   12710: 6a8c         	ld	a1, 0x10(a3)
   12712: cd9d         	beqz	a1, 0x12750 <.Lpcrel_hi12+0xa>
   12714: 6288         	ld	a0, 0x0(a3)
   12716: 864e         	mv	a2, s3
   12718: 86ca         	mv	a3, s2
   1271a: fffff097     	auipc	ra, 0xfffff
   1271e: 414080e7     	jalr	0x414(ra) <__rust_no_alloc_shim_is_unstable+0xffff2a05>
   12722: 00153593     	seqz	a1, a0
   12726: c111         	beqz	a0, 0x1272a <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E+0x32>
   12728: 89aa         	mv	s3, a0
   1272a: 0134b423     	sd	s3, 0x8(s1)
   1272e: 0124b823     	sd	s2, 0x10(s1)
   12732: e08c         	sd	a1, 0x0(s1)
   12734: 70a2         	ld	ra, 0x28(sp)
   12736: 7402         	ld	s0, 0x20(sp)
   12738: 64e2         	ld	s1, 0x18(sp)
   1273a: 6942         	ld	s2, 0x10(sp)
   1273c: 69a2         	ld	s3, 0x8(sp)
   1273e: 6145         	addi	sp, sp, 0x30
   12740: 8082         	ret
   12742: 02090763     	beqz	s2, 0x12770 <.Lpcrel_hi11+0x1c>

0000000000012746 <.Lpcrel_hi12>:
   12746: 0000d517     	auipc	a0, 0xd
   1274a: 9e354003     	lbu	zero, -0x61d(a0)
   1274e: a039         	j	0x1275c <.Lpcrel_hi11+0x8>
   12750: 02090063     	beqz	s2, 0x12770 <.Lpcrel_hi11+0x1c>

0000000000012754 <.Lpcrel_hi11>:
   12754: 0000d517     	auipc	a0, 0xd
   12758: 9d554003     	lbu	zero, -0x62b(a0)
   1275c: 854a         	mv	a0, s2
   1275e: 85ce         	mv	a1, s3
   12760: fffff097     	auipc	ra, 0xfffff
   12764: 384080e7     	jalr	0x384(ra) <__rust_no_alloc_shim_is_unstable+0xffff29bb>
   12768: 00153593     	seqz	a1, a0
   1276c: fd55         	bnez	a0, 0x12728 <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E+0x30>
   1276e: bf75         	j	0x1272a <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E+0x32>
   12770: 854e         	mv	a0, s3
   12772: 0019b593     	seqz	a1, s3
   12776: fa098ae3     	beqz	s3, 0x1272a <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E+0x32>
   1277a: b77d         	j	0x12728 <_ZN5alloc7raw_vec11finish_grow17h53f01316046cb726E+0x30>

000000000001277c <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>:
   1277c: 1141         	addi	sp, sp, -0x10
   1277e: e406         	sd	ra, 0x8(sp)
   12780: e022         	sd	s0, 0x0(sp)
   12782: 0800         	addi	s0, sp, 0x10
   12784: e511         	bnez	a0, 0x12790 <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E+0x14>
   12786: 8532         	mv	a0, a2
   12788: 00000097     	auipc	ra, 0x0
   1278c: e9c080e7     	jalr	-0x164(ra) <_ZN5alloc7raw_vec17capacity_overflow17hf6eb47088a69ee65E>
   12790: 00000097     	auipc	ra, 0x0
   12794: 008080e7     	jalr	0x8(ra) <_ZN5alloc5alloc18handle_alloc_error17h9935f7620e3a2d22E>

0000000000012798 <_ZN5alloc5alloc18handle_alloc_error17h9935f7620e3a2d22E>:
   12798: 1141         	addi	sp, sp, -0x10
   1279a: e406         	sd	ra, 0x8(sp)
   1279c: e022         	sd	s0, 0x0(sp)
   1279e: 0800         	addi	s0, sp, 0x10
   127a0: 862a         	mv	a2, a0
   127a2: 852e         	mv	a0, a1
   127a4: 85b2         	mv	a1, a2
   127a6: fffff097     	auipc	ra, 0xfffff
   127aa: 22c080e7     	jalr	0x22c(ra) <__rust_no_alloc_shim_is_unstable+0xffff28a9>

00000000000127ae <_ZN60_$LT$alloc..string..String$u20$as$u20$core..clone..Clone$GT$5clone17h427ba43c617bed7bE>:
   127ae: 7139         	addi	sp, sp, -0x40
   127b0: fc06         	sd	ra, 0x38(sp)
   127b2: f822         	sd	s0, 0x30(sp)
   127b4: f426         	sd	s1, 0x28(sp)
   127b6: f04a         	sd	s2, 0x20(sp)
   127b8: ec4e         	sd	s3, 0x18(sp)
   127ba: e852         	sd	s4, 0x10(sp)
   127bc: e456         	sd	s5, 0x8(sp)
   127be: 0080         	addi	s0, sp, 0x40
   127c0: 6984         	ld	s1, 0x10(a1)
   127c2: 0404cb63     	bltz	s1, 0x12818 <.Lpcrel_hi110+0x4a>
   127c6: 892a         	mv	s2, a0
   127c8: 0085b983     	ld	s3, 0x8(a1)
   127cc: cc99         	beqz	s1, 0x127ea <.Lpcrel_hi110+0x1c>

00000000000127ce <.Lpcrel_hi110>:
   127ce: 0000d517     	auipc	a0, 0xd
   127d2: 95b54003     	lbu	zero, -0x6a5(a0)
   127d6: 4585         	li	a1, 0x1
   127d8: 4a85         	li	s5, 0x1
   127da: 8526         	mv	a0, s1
   127dc: fffff097     	auipc	ra, 0xfffff
   127e0: 308080e7     	jalr	0x308(ra) <__rust_no_alloc_shim_is_unstable+0xffff29bb>
   127e4: c91d         	beqz	a0, 0x1281a <.Lpcrel_hi111>
   127e6: 8a2a         	mv	s4, a0
   127e8: a011         	j	0x127ec <.Lpcrel_hi110+0x1e>
   127ea: 4a05         	li	s4, 0x1
   127ec: 8552         	mv	a0, s4
   127ee: 85ce         	mv	a1, s3
   127f0: 8626         	mv	a2, s1
   127f2: 00002097     	auipc	ra, 0x2
   127f6: d26080e7     	jalr	-0x2da(ra) <memcpy>
   127fa: 00993023     	sd	s1, 0x0(s2)
   127fe: 01493423     	sd	s4, 0x8(s2)
   12802: 00993823     	sd	s1, 0x10(s2)
   12806: 70e2         	ld	ra, 0x38(sp)
   12808: 7442         	ld	s0, 0x30(sp)
   1280a: 74a2         	ld	s1, 0x28(sp)
   1280c: 7902         	ld	s2, 0x20(sp)
   1280e: 69e2         	ld	s3, 0x18(sp)
   12810: 6a42         	ld	s4, 0x10(sp)
   12812: 6aa2         	ld	s5, 0x8(sp)
   12814: 6121         	addi	sp, sp, 0x40
   12816: 8082         	ret
   12818: 4a81         	li	s5, 0x0

000000000001281a <.Lpcrel_hi111>:
   1281a: 00003517     	auipc	a0, 0x3
   1281e: 4c650613     	addi	a2, a0, 0x4c6
   12822: 8556         	mv	a0, s5
   12824: 85a6         	mv	a1, s1
   12826: 00000097     	auipc	ra, 0x0
   1282a: f56080e7     	jalr	-0xaa(ra) <_ZN5alloc7raw_vec12handle_error17h80da6266efd47034E>

000000000001282e <_ZN68_$LT$core..ptr..alignment..Alignment$u20$as$u20$core..fmt..Debug$GT$3fmt17hbd856da387d08de8E>:
   1282e: 7159         	addi	sp, sp, -0x70
   12830: f486         	sd	ra, 0x68(sp)
   12832: f0a2         	sd	s0, 0x60(sp)
   12834: 1880         	addi	s0, sp, 0x70
   12836: 6108         	ld	a0, 0x0(a0)

0000000000012838 <.Lpcrel_hi200>:
   12838: 00003617     	auipc	a2, 0x3
   1283c: b2063603     	ld	a2, -0x4e0(a2)
   12840: 40a006b3     	neg	a3, a0
   12844: 8ee9         	and	a3, a3, a0
   12846: 02c68633     	mul	a2, a3, a2
   1284a: 9269         	srli	a2, a2, 0x3a

000000000001284c <.Lpcrel_hi201>:
   1284c: 00003697     	auipc	a3, 0x3
   12850: 4ac68693     	addi	a3, a3, 0x4ac
   12854: 9636         	add	a2, a2, a3
   12856: 00064603     	lbu	a2, 0x0(a2)
   1285a: fea43023     	sd	a0, -0x20(s0)
   1285e: fec42623     	sw	a2, -0x14(s0)
   12862: fe040513     	addi	a0, s0, -0x20
   12866: fca43023     	sd	a0, -0x40(s0)

000000000001286a <.Lpcrel_hi202>:
   1286a: 00001517     	auipc	a0, 0x1
   1286e: 50650513     	addi	a0, a0, 0x506
   12872: fca43423     	sd	a0, -0x38(s0)
   12876: fec40513     	addi	a0, s0, -0x14
   1287a: fca43823     	sd	a0, -0x30(s0)

000000000001287e <.Lpcrel_hi203>:
   1287e: 00001517     	auipc	a0, 0x1
   12882: 71850513     	addi	a0, a0, 0x718
   12886: fca43c23     	sd	a0, -0x28(s0)

000000000001288a <.Lpcrel_hi204>:
   1288a: 00003517     	auipc	a0, 0x3
   1288e: 4b650513     	addi	a0, a0, 0x4b6
   12892: f8a43823     	sd	a0, -0x70(s0)
   12896: 450d         	li	a0, 0x3
   12898: f8a43c23     	sd	a0, -0x68(s0)
   1289c: fa043823     	sd	zero, -0x50(s0)
   128a0: fc040613     	addi	a2, s0, -0x40
   128a4: 7988         	ld	a0, 0x30(a1)
   128a6: 7d8c         	ld	a1, 0x38(a1)
   128a8: fac43023     	sd	a2, -0x60(s0)
   128ac: 4609         	li	a2, 0x2
   128ae: fac43423     	sd	a2, -0x58(s0)
   128b2: f9040613     	addi	a2, s0, -0x70
   128b6: 00000097     	auipc	ra, 0x0
   128ba: 6f4080e7     	jalr	0x6f4(ra) <_ZN4core3fmt5write17h1b882e4f6891aa5dE>
   128be: 70a6         	ld	ra, 0x68(sp)
   128c0: 7406         	ld	s0, 0x60(sp)
   128c2: 6165         	addi	sp, sp, 0x70
   128c4: 8082         	ret

00000000000128c6 <_ZN4core6option13unwrap_failed17h78203addebfbdbebE>:
   128c6: 1141         	addi	sp, sp, -0x10
   128c8: e406         	sd	ra, 0x8(sp)
   128ca: e022         	sd	s0, 0x0(sp)
   128cc: 0800         	addi	s0, sp, 0x10
   128ce: 862a         	mv	a2, a0

00000000000128d0 <.Lpcrel_hi331>:
   128d0: 00003517     	auipc	a0, 0x3
   128d4: 4a050513     	addi	a0, a0, 0x4a0
   128d8: 02b00593     	li	a1, 0x2b
   128dc: 00000097     	auipc	ra, 0x0
   128e0: 07e080e7     	jalr	0x7e(ra) <_ZN4core9panicking5panic17h6952156bbcf3c8fdE>

00000000000128e4 <_ZN4core6option13expect_failed17he7ca34d564694790E>:
   128e4: 711d         	addi	sp, sp, -0x60
   128e6: ec86         	sd	ra, 0x58(sp)
   128e8: e8a2         	sd	s0, 0x50(sp)
   128ea: 1080         	addi	s0, sp, 0x60
   128ec: faa43023     	sd	a0, -0x60(s0)
   128f0: fab43423     	sd	a1, -0x58(s0)
   128f4: fa040513     	addi	a0, s0, -0x60
   128f8: fea43023     	sd	a0, -0x20(s0)

00000000000128fc <.Lpcrel_hi332>:
   128fc: 00002517     	auipc	a0, 0x2
   12900: b3e50513     	addi	a0, a0, -0x4c2
   12904: fea43423     	sd	a0, -0x18(s0)

0000000000012908 <.Lpcrel_hi333>:
   12908: 00003517     	auipc	a0, 0x3
   1290c: 8b050513     	addi	a0, a0, -0x750
   12910: faa43823     	sd	a0, -0x50(s0)
   12914: 4505         	li	a0, 0x1
   12916: faa43c23     	sd	a0, -0x48(s0)
   1291a: fc043823     	sd	zero, -0x30(s0)
   1291e: fe040593     	addi	a1, s0, -0x20
   12922: fcb43023     	sd	a1, -0x40(s0)
   12926: fca43423     	sd	a0, -0x38(s0)
   1292a: fb040513     	addi	a0, s0, -0x50
   1292e: 85b2         	mv	a1, a2
   12930: 00000097     	auipc	ra, 0x0
   12934: 008080e7     	jalr	0x8(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

0000000000012938 <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>:
   12938: 7179         	addi	sp, sp, -0x30
   1293a: f406         	sd	ra, 0x28(sp)
   1293c: f022         	sd	s0, 0x20(sp)
   1293e: 1800         	addi	s0, sp, 0x30
   12940: fca43c23     	sd	a0, -0x28(s0)
   12944: feb43023     	sd	a1, -0x20(s0)
   12948: 4505         	li	a0, 0x1
   1294a: fea41423     	sh	a0, -0x18(s0)
   1294e: fd840513     	addi	a0, s0, -0x28
   12952: fffff097     	auipc	ra, 0xfffff
   12956: 6d2080e7     	jalr	0x6d2(ra) <__rust_no_alloc_shim_is_unstable+0xffff2efb>

000000000001295a <_ZN4core9panicking5panic17h6952156bbcf3c8fdE>:
   1295a: 715d         	addi	sp, sp, -0x50
   1295c: e486         	sd	ra, 0x48(sp)
   1295e: e0a2         	sd	s0, 0x40(sp)
   12960: 0880         	addi	s0, sp, 0x50
   12962: fea43023     	sd	a0, -0x20(s0)
   12966: feb43423     	sd	a1, -0x18(s0)
   1296a: fe040513     	addi	a0, s0, -0x20
   1296e: faa43823     	sd	a0, -0x50(s0)
   12972: 4505         	li	a0, 0x1
   12974: faa43c23     	sd	a0, -0x48(s0)
   12978: fc043823     	sd	zero, -0x30(s0)
   1297c: 4521         	li	a0, 0x8
   1297e: fca43023     	sd	a0, -0x40(s0)
   12982: fc043423     	sd	zero, -0x38(s0)
   12986: fb040513     	addi	a0, s0, -0x50
   1298a: 85b2         	mv	a1, a2
   1298c: 00000097     	auipc	ra, 0x0
   12990: fac080e7     	jalr	-0x54(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

0000000000012994 <_ZN4core9panicking18panic_bounds_check17hcdbdf68ae840fc45E>:
   12994: 7159         	addi	sp, sp, -0x70
   12996: f486         	sd	ra, 0x68(sp)
   12998: f0a2         	sd	s0, 0x60(sp)
   1299a: 1880         	addi	s0, sp, 0x70
   1299c: f0040693     	addi	a3, s0, -0x100
   129a0: eac8         	sd	a0, 0x90(a3)
   129a2: eecc         	sd	a1, 0x98(a3)
   129a4: f9840513     	addi	a0, s0, -0x68
   129a8: eae8         	sd	a0, 0xd0(a3)

00000000000129aa <.Lpcrel_hi344>:
   129aa: 00002517     	auipc	a0, 0x2
   129ae: 8d050513     	addi	a0, a0, -0x730
   129b2: eee8         	sd	a0, 0xd8(a3)
   129b4: f9040593     	addi	a1, s0, -0x70
   129b8: f2ec         	sd	a1, 0xe0(a3)
   129ba: f6e8         	sd	a0, 0xe8(a3)

00000000000129bc <.Lpcrel_hi345>:
   129bc: 00003517     	auipc	a0, 0x3
   129c0: 3f450513     	addi	a0, a0, 0x3f4
   129c4: f2c8         	sd	a0, 0xa0(a3)
   129c6: 4509         	li	a0, 0x2
   129c8: f6c8         	sd	a0, 0xa8(a3)
   129ca: fc043023     	sd	zero, -0x40(s0)
   129ce: fd040593     	addi	a1, s0, -0x30
   129d2: facc         	sd	a1, 0xb0(a3)
   129d4: fec8         	sd	a0, 0xb8(a3)
   129d6: fa040513     	addi	a0, s0, -0x60
   129da: 85b2         	mv	a1, a2
   129dc: 00000097     	auipc	ra, 0x0
   129e0: f5c080e7     	jalr	-0xa4(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

00000000000129e4 <_ZN4core9panicking19assert_failed_inner17hc4a20c2ec30e9d4dE>:
   129e4: 7151         	addi	sp, sp, -0xf0
   129e6: f586         	sd	ra, 0xe8(sp)
   129e8: f1a2         	sd	s0, 0xe0(sp)
   129ea: eda6         	sd	s1, 0xd8(sp)
   129ec: e9ca         	sd	s2, 0xd0(sp)
   129ee: 1980         	addi	s0, sp, 0xf0
   129f0: 84c2         	mv	s1, a6
   129f2: f0b43823     	sd	a1, -0xf0(s0)
   129f6: f0040593     	addi	a1, s0, -0x100
   129fa: ed90         	sd	a2, 0x18(a1)
   129fc: f194         	sd	a3, 0x20(a1)
   129fe: f598         	sd	a4, 0x28(a1)
   12a00: c909         	beqz	a0, 0x12a12 <.Lpcrel_hi355>
   12a02: 4585         	li	a1, 0x1
   12a04: 00b51f63     	bne	a0, a1, 0x12a22 <.Lpcrel_hi357>

0000000000012a08 <.Lpcrel_hi356>:
   12a08: 00003517     	auipc	a0, 0x3
   12a0c: 3ca50513     	addi	a0, a0, 0x3ca
   12a10: a029         	j	0x12a1a <.Lpcrel_hi355+0x8>

0000000000012a12 <.Lpcrel_hi355>:
   12a12: 00003517     	auipc	a0, 0x3
   12a16: 3be50513     	addi	a0, a0, 0x3be
   12a1a: f2a43823     	sd	a0, -0xd0(s0)
   12a1e: 4509         	li	a0, 0x2
   12a20: a801         	j	0x12a30 <.Lpcrel_hi357+0xe>

0000000000012a22 <.Lpcrel_hi357>:
   12a22: 00003517     	auipc	a0, 0x3
   12a26: 3b250513     	addi	a0, a0, 0x3b2
   12a2a: f2a43823     	sd	a0, -0xd0(s0)
   12a2e: 451d         	li	a0, 0x7
   12a30: 638c         	ld	a1, 0x0(a5)
   12a32: f2a43c23     	sd	a0, -0xc8(s0)
   12a36: ed95         	bnez	a1, 0x12a72 <.Lpcrel_hi360+0xe>
   12a38: f3040513     	addi	a0, s0, -0xd0
   12a3c: f0040613     	addi	a2, s0, -0x100
   12a40: fa28         	sd	a0, 0x70(a2)

0000000000012a42 <.Lpcrel_hi358>:
   12a42: 00002517     	auipc	a0, 0x2
   12a46: 9f850513     	addi	a0, a0, -0x608
   12a4a: fe28         	sd	a0, 0x78(a2)
   12a4c: f1040513     	addi	a0, s0, -0xf0
   12a50: e248         	sd	a0, 0x80(a2)

0000000000012a52 <.Lpcrel_hi359>:
   12a52: 00002517     	auipc	a0, 0x2
   12a56: 9d250513     	addi	a0, a0, -0x62e
   12a5a: e648         	sd	a0, 0x88(a2)
   12a5c: f2040593     	addi	a1, s0, -0xe0
   12a60: ea4c         	sd	a1, 0x90(a2)
   12a62: ee48         	sd	a0, 0x98(a2)

0000000000012a64 <.Lpcrel_hi360>:
   12a64: 00003517     	auipc	a0, 0x3
   12a68: 39c50513     	addi	a0, a0, 0x39c
   12a6c: fa48         	sd	a0, 0xb0(a2)
   12a6e: 450d         	li	a0, 0x3
   12a70: a8b9         	j	0x12ace <.Lpcrel_hi364+0xc>
   12a72: f4040513     	addi	a0, s0, -0xc0
   12a76: 03000613     	li	a2, 0x30
   12a7a: f4040913     	addi	s2, s0, -0xc0
   12a7e: 85be         	mv	a1, a5
   12a80: 00002097     	auipc	ra, 0x2
   12a84: a98080e7     	jalr	-0x568(ra) <memcpy>
   12a88: f3040513     	addi	a0, s0, -0xd0
   12a8c: f0040613     	addi	a2, s0, -0x100
   12a90: fa28         	sd	a0, 0x70(a2)

0000000000012a92 <.Lpcrel_hi361>:
   12a92: 00002517     	auipc	a0, 0x2
   12a96: 9a850513     	addi	a0, a0, -0x658
   12a9a: fe28         	sd	a0, 0x78(a2)
   12a9c: f9243023     	sd	s2, -0x80(s0)

0000000000012aa0 <.Lpcrel_hi362>:
   12aa0: 00000517     	auipc	a0, 0x0
   12aa4: 4ea50513     	addi	a0, a0, 0x4ea
   12aa8: e648         	sd	a0, 0x88(a2)
   12aaa: f1040513     	addi	a0, s0, -0xf0
   12aae: ea48         	sd	a0, 0x90(a2)

0000000000012ab0 <.Lpcrel_hi363>:
   12ab0: 00002517     	auipc	a0, 0x2
   12ab4: 97450513     	addi	a0, a0, -0x68c
   12ab8: ee48         	sd	a0, 0x98(a2)
   12aba: f2040593     	addi	a1, s0, -0xe0
   12abe: f24c         	sd	a1, 0xa0(a2)
   12ac0: f648         	sd	a0, 0xa8(a2)

0000000000012ac2 <.Lpcrel_hi364>:
   12ac2: 00003517     	auipc	a0, 0x3
   12ac6: 37e50513     	addi	a0, a0, 0x37e
   12aca: fa48         	sd	a0, 0xb0(a2)
   12acc: 4511         	li	a0, 0x4
   12ace: f0040613     	addi	a2, s0, -0x100
   12ad2: fe48         	sd	a0, 0xb8(a2)
   12ad4: fc043823     	sd	zero, -0x30(s0)
   12ad8: f7040593     	addi	a1, s0, -0x90
   12adc: e26c         	sd	a1, 0xc0(a2)
   12ade: e668         	sd	a0, 0xc8(a2)
   12ae0: fb040513     	addi	a0, s0, -0x50
   12ae4: 85a6         	mv	a1, s1
   12ae6: 00000097     	auipc	ra, 0x0
   12aea: e52080e7     	jalr	-0x1ae(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

0000000000012aee <_ZN4core6result13unwrap_failed17h097de9be360fa68cE>:
   12aee: 7119         	addi	sp, sp, -0x80
   12af0: fc86         	sd	ra, 0x78(sp)
   12af2: f8a2         	sd	s0, 0x70(sp)
   12af4: 0100         	addi	s0, sp, 0x80
   12af6: f8a43023     	sd	a0, -0x80(s0)
   12afa: f8b43423     	sd	a1, -0x78(s0)
   12afe: f8c43823     	sd	a2, -0x70(s0)
   12b02: f8d43c23     	sd	a3, -0x68(s0)
   12b06: f8040513     	addi	a0, s0, -0x80
   12b0a: fca43823     	sd	a0, -0x30(s0)

0000000000012b0e <.Lpcrel_hi365>:
   12b0e: 00002517     	auipc	a0, 0x2
   12b12: 92c50513     	addi	a0, a0, -0x6d4
   12b16: fca43c23     	sd	a0, -0x28(s0)
   12b1a: f9040513     	addi	a0, s0, -0x70
   12b1e: fea43023     	sd	a0, -0x20(s0)

0000000000012b22 <.Lpcrel_hi366>:
   12b22: 00002517     	auipc	a0, 0x2
   12b26: 90250513     	addi	a0, a0, -0x6fe
   12b2a: fea43423     	sd	a0, -0x18(s0)

0000000000012b2e <.Lpcrel_hi367>:
   12b2e: 00003517     	auipc	a0, 0x3
   12b32: 35a50513     	addi	a0, a0, 0x35a
   12b36: faa43023     	sd	a0, -0x60(s0)
   12b3a: 4509         	li	a0, 0x2
   12b3c: faa43423     	sd	a0, -0x58(s0)
   12b40: fc043023     	sd	zero, -0x40(s0)
   12b44: fd040593     	addi	a1, s0, -0x30
   12b48: fab43823     	sd	a1, -0x50(s0)
   12b4c: faa43c23     	sd	a0, -0x48(s0)
   12b50: fa040513     	addi	a0, s0, -0x60
   12b54: 85ba         	mv	a1, a4
   12b56: 00000097     	auipc	ra, 0x0
   12b5a: de2080e7     	jalr	-0x21e(ra) <_ZN4core9panicking9panic_fmt17h00a5340a3f228c5eE>

0000000000012b5e <_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17hc2d1553be630419cE>:
   12b5e: 7135         	addi	sp, sp, -0xa0
   12b60: ed06         	sd	ra, 0x98(sp)
   12b62: e922         	sd	s0, 0x90(sp)
   12b64: e526         	sd	s1, 0x88(sp)
   12b66: e14a         	sd	s2, 0x80(sp)
   12b68: fcce         	sd	s3, 0x78(sp)
   12b6a: f8d2         	sd	s4, 0x70(sp)
   12b6c: f4d6         	sd	s5, 0x68(sp)
   12b6e: f0da         	sd	s6, 0x60(sp)
   12b70: ecde         	sd	s7, 0x58(sp)
   12b72: e8e2         	sd	s8, 0x50(sp)
   12b74: e4e6         	sd	s9, 0x48(sp)
   12b76: e0ea         	sd	s10, 0x40(sp)
   12b78: fc6e         	sd	s11, 0x38(sp)
   12b7a: 1100         	addi	s0, sp, 0xa0
   12b7c: 89b2         	mv	s3, a2
   12b7e: 892e         	mv	s2, a1
   12b80: 4d01         	li	s10, 0x0
   12b82: 4d81         	li	s11, 0x0
   12b84: 4c81         	li	s9, 0x0
   12b86: 0a0a15b7     	lui	a1, 0xa0a1
   12b8a: a0a58b1b     	addiw	s6, a1, -0x5f6
   12b8e: 020b1593     	slli	a1, s6, 0x20
   12b92: 9b2e         	add	s6, s6, a1

0000000000012b94 <.Lpcrel_hi371>:
   12b94: 00002597     	auipc	a1, 0x2
   12b98: 7bc5bb83     	ld	s7, 0x7bc(a1)
   12b9c: 6904         	ld	s1, 0x10(a0)
   12b9e: 610c         	ld	a1, 0x0(a0)
   12ba0: f8b43823     	sd	a1, -0x70(s0)
   12ba4: 6508         	ld	a0, 0x8(a0)
   12ba6: f8a43423     	sd	a0, -0x78(s0)
   12baa: fff90513     	addi	a0, s2, -0x1
   12bae: f6a43823     	sd	a0, -0x90(s0)
   12bb2: 40c00533     	neg	a0, a2
   12bb6: f8a43023     	sd	a0, -0x80(s0)

0000000000012bba <.Lpcrel_hi372>:
   12bba: 00003517     	auipc	a0, 0x3
   12bbe: 8ca50513     	addi	a0, a0, -0x736
   12bc2: f6a43c23     	sd	a0, -0x88(s0)
   12bc6: 4a29         	li	s4, 0xa
   12bc8: f6943423     	sd	s1, -0x98(s0)
   12bcc: a805         	j	0x12bfc <.Lpcrel_hi372+0x42>
   12bce: f7043503     	ld	a0, -0x90(s0)
   12bd2: 9556         	add	a0, a0, s5
   12bd4: 00054503     	lbu	a0, 0x0(a0)
   12bd8: 1559         	addi	a0, a0, -0xa
   12bda: 00153513     	seqz	a0, a0
   12bde: 00a48023     	sb	a0, 0x0(s1)
   12be2: f8843503     	ld	a0, -0x78(s0)
   12be6: 6d14         	ld	a3, 0x18(a0)
   12be8: 41ba8633     	sub	a2, s5, s11
   12bec: 01b905b3     	add	a1, s2, s11
   12bf0: f9043503     	ld	a0, -0x70(s0)
   12bf4: 9682         	jalr	a3
   12bf6: 8de2         	mv	s11, s8
   12bf8: 14051263     	bnez	a0, 0x12d3c <.Lpcrel_hi373+0xa6>
   12bfc: 001cf513     	andi	a0, s9, 0x1
   12c00: 12051c63     	bnez	a0, 0x12d38 <.Lpcrel_hi373+0xa2>
   12c04: 01a9f763     	bgeu	s3, s10, 0x12c12 <.Lpcrel_hi372+0x58>
   12c08: 8c6a         	mv	s8, s10
   12c0a: a8e5         	j	0x12d02 <.Lpcrel_hi373+0x6c>
   12c0c: 8d62         	mv	s10, s8
   12c0e: 0f89ea63     	bltu	s3, s8, 0x12d02 <.Lpcrel_hi373+0x6c>
   12c12: 41a98833     	sub	a6, s3, s10
   12c16: 01a90533     	add	a0, s2, s10
   12c1a: 45bd         	li	a1, 0xf
   12c1c: 0305e163     	bltu	a1, a6, 0x12c3e <.Lpcrel_hi372+0x84>
   12c20: 0fa98063     	beq	s3, s10, 0x12d00 <.Lpcrel_hi373+0x6a>
   12c24: 4581         	li	a1, 0x0
   12c26: f8043603     	ld	a2, -0x80(s0)
   12c2a: 966a         	add	a2, a2, s10
   12c2c: 00054683     	lbu	a3, 0x0(a0)
   12c30: 03468c63     	beq	a3, s4, 0x12c68 <.Lpcrel_hi372+0xae>
   12c34: 15fd         	addi	a1, a1, -0x1
   12c36: 0505         	addi	a0, a0, 0x1
   12c38: feb61ae3     	bne	a2, a1, 0x12c2c <.Lpcrel_hi372+0x72>
   12c3c: a0d1         	j	0x12d00 <.Lpcrel_hi373+0x6a>
   12c3e: 00750713     	addi	a4, a0, 0x7
   12c42: 9b61         	andi	a4, a4, -0x8
   12c44: 40a70633     	sub	a2, a4, a0
   12c48: ca0d         	beqz	a2, 0x12c7a <.Lpcrel_hi372+0xc0>
   12c4a: 4681         	li	a3, 0x0
   12c4c: 00d505b3     	add	a1, a0, a3
   12c50: 0005c583     	lbu	a1, 0x0(a1)
   12c54: 01458c63     	beq	a1, s4, 0x12c6c <.Lpcrel_hi372+0xb2>
   12c58: 0685         	addi	a3, a3, 0x1
   12c5a: fed619e3     	bne	a2, a3, 0x12c4c <.Lpcrel_hi372+0x92>
   12c5e: ff080893     	addi	a7, a6, -0x10
   12c62: 00c8fe63     	bgeu	a7, a2, 0x12c7e <.Lpcrel_hi372+0xc4>
   12c66: a0b9         	j	0x12cb4 <.Lpcrel_hi373+0x1e>
   12c68: 40b006b3     	neg	a3, a1
   12c6c: 00dd0533     	add	a0, s10, a3
   12c70: 00150c13     	addi	s8, a0, 0x1
   12c74: f9357ce3     	bgeu	a0, s3, 0x12c0c <.Lpcrel_hi372+0x52>
   12c78: a895         	j	0x12cec <.Lpcrel_hi373+0x56>
   12c7a: ff080893     	addi	a7, a6, -0x10
   12c7e: 45a1         	li	a1, 0x8
   12c80: 972e         	add	a4, a4, a1
   12c82: ff873783     	ld	a5, -0x8(a4)
   12c86: 630c         	ld	a1, 0x0(a4)
   12c88: 0167c4b3     	xor	s1, a5, s6
   12c8c: 409b84b3     	sub	s1, s7, s1
   12c90: 8fc5         	or	a5, a5, s1
   12c92: 0165c5b3     	xor	a1, a1, s6

0000000000012c96 <.Lpcrel_hi373>:
   12c96: 00002497     	auipc	s1, 0x2
   12c9a: 6b24b483     	ld	s1, 0x6b2(s1)
   12c9e: 40bb86b3     	sub	a3, s7, a1
   12ca2: 8dd5         	or	a1, a1, a3
   12ca4: 8dfd         	and	a1, a1, a5
   12ca6: 8de5         	and	a1, a1, s1
   12ca8: 00959663     	bne	a1, s1, 0x12cb4 <.Lpcrel_hi373+0x1e>
   12cac: 0641         	addi	a2, a2, 0x10
   12cae: 0741         	addi	a4, a4, 0x10
   12cb0: fcc8f9e3     	bgeu	a7, a2, 0x12c82 <.Lpcrel_hi372+0xc8>
   12cb4: 07060e63     	beq	a2, a6, 0x12d30 <.Lpcrel_hi373+0x9a>
   12cb8: 00c505b3     	add	a1, a0, a2
   12cbc: 40c00533     	neg	a0, a2
   12cc0: f8043603     	ld	a2, -0x80(s0)
   12cc4: 966a         	add	a2, a2, s10
   12cc6: f6843483     	ld	s1, -0x98(s0)
   12cca: 0005c683     	lbu	a3, 0x0(a1)
   12cce: 01468763     	beq	a3, s4, 0x12cdc <.Lpcrel_hi373+0x46>
   12cd2: 157d         	addi	a0, a0, -0x1
   12cd4: 0585         	addi	a1, a1, 0x1
   12cd6: fea61ae3     	bne	a2, a0, 0x12cca <.Lpcrel_hi373+0x34>
   12cda: a01d         	j	0x12d00 <.Lpcrel_hi373+0x6a>
   12cdc: 40a006b3     	neg	a3, a0
   12ce0: 00dd0533     	add	a0, s10, a3
   12ce4: 00150c13     	addi	s8, a0, 0x1
   12ce8: f33572e3     	bgeu	a0, s3, 0x12c0c <.Lpcrel_hi372+0x52>
   12cec: 9d4a         	add	s10, s10, s2
   12cee: 96ea         	add	a3, a3, s10
   12cf0: 0006c503     	lbu	a0, 0x0(a3)
   12cf4: f1451ce3     	bne	a0, s4, 0x12c0c <.Lpcrel_hi372+0x52>
   12cf8: 4c81         	li	s9, 0x0
   12cfa: 8d62         	mv	s10, s8
   12cfc: 8ae2         	mv	s5, s8
   12cfe: a801         	j	0x12d0e <.Lpcrel_hi373+0x78>
   12d00: 8c4e         	mv	s8, s3
   12d02: 4c85         	li	s9, 0x1
   12d04: 8d62         	mv	s10, s8
   12d06: 8c6e         	mv	s8, s11
   12d08: 8ace         	mv	s5, s3
   12d0a: 033d8763     	beq	s11, s3, 0x12d38 <.Lpcrel_hi373+0xa2>
   12d0e: 0004c503     	lbu	a0, 0x0(s1)
   12d12: c919         	beqz	a0, 0x12d28 <.Lpcrel_hi373+0x92>
   12d14: f8843503     	ld	a0, -0x78(s0)
   12d18: 6d14         	ld	a3, 0x18(a0)
   12d1a: 4611         	li	a2, 0x4
   12d1c: f9043503     	ld	a0, -0x70(s0)
   12d20: f7843583     	ld	a1, -0x88(s0)
   12d24: 9682         	jalr	a3
   12d26: e919         	bnez	a0, 0x12d3c <.Lpcrel_hi373+0xa6>
   12d28: ebba93e3     	bne	s5, s11, 0x12bce <.Lpcrel_hi372+0x14>
   12d2c: 4501         	li	a0, 0x0
   12d2e: bd45         	j	0x12bde <.Lpcrel_hi372+0x24>
   12d30: 8c4e         	mv	s8, s3
   12d32: f6843483     	ld	s1, -0x98(s0)
   12d36: b7f1         	j	0x12d02 <.Lpcrel_hi373+0x6c>
   12d38: 4501         	li	a0, 0x0
   12d3a: a011         	j	0x12d3e <.Lpcrel_hi373+0xa8>
   12d3c: 4505         	li	a0, 0x1
   12d3e: 60ea         	ld	ra, 0x98(sp)
   12d40: 644a         	ld	s0, 0x90(sp)
   12d42: 64aa         	ld	s1, 0x88(sp)
   12d44: 690a         	ld	s2, 0x80(sp)
   12d46: 79e6         	ld	s3, 0x78(sp)
   12d48: 7a46         	ld	s4, 0x70(sp)
   12d4a: 7aa6         	ld	s5, 0x68(sp)
   12d4c: 7b06         	ld	s6, 0x60(sp)
   12d4e: 6be6         	ld	s7, 0x58(sp)
   12d50: 6c46         	ld	s8, 0x50(sp)
   12d52: 6ca6         	ld	s9, 0x48(sp)
   12d54: 6d06         	ld	s10, 0x40(sp)
   12d56: 7de2         	ld	s11, 0x38(sp)
   12d58: 610d         	addi	sp, sp, 0xa0
   12d5a: 8082         	ret

0000000000012d5c <_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$10write_char17h6a3baeed730f45e8E>:
   12d5c: 7179         	addi	sp, sp, -0x30
   12d5e: f406         	sd	ra, 0x28(sp)
   12d60: f022         	sd	s0, 0x20(sp)
   12d62: ec26         	sd	s1, 0x18(sp)
   12d64: e84a         	sd	s2, 0x10(sp)
   12d66: e44e         	sd	s3, 0x8(sp)
   12d68: e052         	sd	s4, 0x0(sp)
   12d6a: 1800         	addi	s0, sp, 0x30
   12d6c: 6904         	ld	s1, 0x10(a0)
   12d6e: 0004c603     	lbu	a2, 0x0(s1)
   12d72: 00053903     	ld	s2, 0x0(a0)
   12d76: 00853983     	ld	s3, 0x8(a0)
   12d7a: c61d         	beqz	a2, 0x12da8 <.Lpcrel_hi374+0x28>
   12d7c: 0189b703     	ld	a4, 0x18(s3)

0000000000012d80 <.Lpcrel_hi374>:
   12d80: 00002517     	auipc	a0, 0x2
   12d84: 70450693     	addi	a3, a0, 0x704
   12d88: 4611         	li	a2, 0x4
   12d8a: 854a         	mv	a0, s2
   12d8c: 8a2e         	mv	s4, a1
   12d8e: 85b6         	mv	a1, a3
   12d90: 9702         	jalr	a4
   12d92: 85d2         	mv	a1, s4
   12d94: c911         	beqz	a0, 0x12da8 <.Lpcrel_hi374+0x28>
   12d96: 4505         	li	a0, 0x1
   12d98: 70a2         	ld	ra, 0x28(sp)
   12d9a: 7402         	ld	s0, 0x20(sp)
   12d9c: 64e2         	ld	s1, 0x18(sp)
   12d9e: 6942         	ld	s2, 0x10(sp)
   12da0: 69a2         	ld	s3, 0x8(sp)
   12da2: 6a02         	ld	s4, 0x0(sp)
   12da4: 6145         	addi	sp, sp, 0x30
   12da6: 8082         	ret
   12da8: ff658513     	addi	a0, a1, -0xa
   12dac: 00153513     	seqz	a0, a0
   12db0: 00a48023     	sb	a0, 0x0(s1)
   12db4: 0209b783     	ld	a5, 0x20(s3)
   12db8: 854a         	mv	a0, s2
   12dba: 70a2         	ld	ra, 0x28(sp)
   12dbc: 7402         	ld	s0, 0x20(sp)
   12dbe: 64e2         	ld	s1, 0x18(sp)
   12dc0: 6942         	ld	s2, 0x10(sp)
   12dc2: 69a2         	ld	s3, 0x8(sp)
   12dc4: 6a02         	ld	s4, 0x0(sp)
   12dc6: 6145         	addi	sp, sp, 0x30
   12dc8: 8782         	jr	a5

0000000000012dca <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE>:
   12dca: 7171         	addi	sp, sp, -0xb0
   12dcc: f506         	sd	ra, 0xa8(sp)
   12dce: f122         	sd	s0, 0xa0(sp)
   12dd0: ed26         	sd	s1, 0x98(sp)
   12dd2: e94a         	sd	s2, 0x90(sp)
   12dd4: e54e         	sd	s3, 0x88(sp)
   12dd6: e152         	sd	s4, 0x80(sp)
   12dd8: fcd6         	sd	s5, 0x78(sp)
   12dda: f8da         	sd	s6, 0x70(sp)
   12ddc: f4de         	sd	s7, 0x68(sp)
   12dde: f0e2         	sd	s8, 0x60(sp)
   12de0: 1900         	addi	s0, sp, 0xb0
   12de2: 8aaa         	mv	s5, a0
   12de4: 00854503     	lbu	a0, 0x8(a0)
   12de8: 4b05         	li	s6, 0x1
   12dea: 4485         	li	s1, 0x1
   12dec: c115         	beqz	a0, 0x12e10 <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x46>
   12dee: 009a8423     	sb	s1, 0x8(s5)
   12df2: 016a84a3     	sb	s6, 0x9(s5)
   12df6: 8556         	mv	a0, s5
   12df8: 70aa         	ld	ra, 0xa8(sp)
   12dfa: 740a         	ld	s0, 0xa0(sp)
   12dfc: 64ea         	ld	s1, 0x98(sp)
   12dfe: 694a         	ld	s2, 0x90(sp)
   12e00: 69aa         	ld	s3, 0x88(sp)
   12e02: 6a0a         	ld	s4, 0x80(sp)
   12e04: 7ae6         	ld	s5, 0x78(sp)
   12e06: 7b46         	ld	s6, 0x70(sp)
   12e08: 7ba6         	ld	s7, 0x68(sp)
   12e0a: 7c06         	ld	s8, 0x60(sp)
   12e0c: 614d         	addi	sp, sp, 0xb0
   12e0e: 8082         	ret
   12e10: 89ba         	mv	s3, a4
   12e12: 8936         	mv	s2, a3
   12e14: 000aba03     	ld	s4, 0x0(s5)
   12e18: 024a4503     	lbu	a0, 0x24(s4)
   12e1c: 009ac683     	lbu	a3, 0x9(s5)
   12e20: 8911         	andi	a0, a0, 0x4
   12e22: e909         	bnez	a0, 0x12e34 <.Lpcrel_hi376+0xa>
   12e24: 8bae         	mv	s7, a1
   12e26: 8c32         	mv	s8, a2
   12e28: e2e5         	bnez	a3, 0x12f08 <.Lpcrel_hi375>

0000000000012e2a <.Lpcrel_hi376>:
   12e2a: 00003517     	auipc	a0, 0x3
   12e2e: 0ae50593     	addi	a1, a0, 0xae
   12e32: a8f9         	j	0x12f10 <.Lpcrel_hi375+0x8>
   12e34: e29d         	bnez	a3, 0x12e5a <.Lpcrel_hi378+0x1a>
   12e36: 038a3683     	ld	a3, 0x38(s4)
   12e3a: 030a3503     	ld	a0, 0x30(s4)
   12e3e: 6e98         	ld	a4, 0x18(a3)

0000000000012e40 <.Lpcrel_hi378>:
   12e40: 00003697     	auipc	a3, 0x3
   12e44: 09d68693     	addi	a3, a3, 0x9d
   12e48: 8bb2         	mv	s7, a2
   12e4a: 460d         	li	a2, 0x3
   12e4c: 84ae         	mv	s1, a1
   12e4e: 85b6         	mv	a1, a3
   12e50: 9702         	jalr	a4
   12e52: 85a6         	mv	a1, s1
   12e54: 865e         	mv	a2, s7
   12e56: 4485         	li	s1, 0x1
   12e58: f959         	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>
   12e5a: 000a3503     	ld	a0, 0x0(s4)
   12e5e: f6a43823     	sd	a0, -0x90(s0)
   12e62: 008a3503     	ld	a0, 0x8(s4)
   12e66: f6a43c23     	sd	a0, -0x88(s0)
   12e6a: 010a3503     	ld	a0, 0x10(s4)
   12e6e: f8a43023     	sd	a0, -0x80(s0)
   12e72: 018a3503     	ld	a0, 0x18(s4)
   12e76: 030a3683     	ld	a3, 0x30(s4)
   12e7a: 4485         	li	s1, 0x1
   12e7c: f8a43423     	sd	a0, -0x78(s0)
   12e80: 020a3503     	ld	a0, 0x20(s4)
   12e84: f4d43823     	sd	a3, -0xb0(s0)
   12e88: 038a3683     	ld	a3, 0x38(s4)
   12e8c: f69407a3     	sb	s1, -0x91(s0)
   12e90: f8a43823     	sd	a0, -0x70(s0)
   12e94: 028a3503     	ld	a0, 0x28(s4)
   12e98: f4d43c23     	sd	a3, -0xa8(s0)
   12e9c: f6f40693     	addi	a3, s0, -0x91
   12ea0: f6d43023     	sd	a3, -0xa0(s0)
   12ea4: f8a43c23     	sd	a0, -0x68(s0)
   12ea8: f5040513     	addi	a0, s0, -0xb0
   12eac: faa43023     	sd	a0, -0x60(s0)

0000000000012eb0 <.Lpcrel_hi379>:
   12eb0: 00003517     	auipc	a0, 0x3
   12eb4: ff850513     	addi	a0, a0, -0x8
   12eb8: faa43423     	sd	a0, -0x58(s0)
   12ebc: f5040513     	addi	a0, s0, -0xb0
   12ec0: 00000097     	auipc	ra, 0x0
   12ec4: c9e080e7     	jalr	-0x362(ra) <_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17hc2d1553be630419cE>
   12ec8: f11d         	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>

0000000000012eca <.Lpcrel_hi380>:
   12eca: 00003517     	auipc	a0, 0x3
   12ece: fb650593     	addi	a1, a0, -0x4a
   12ed2: f5040513     	addi	a0, s0, -0xb0
   12ed6: 4609         	li	a2, 0x2
   12ed8: 00000097     	auipc	ra, 0x0
   12edc: c86080e7     	jalr	-0x37a(ra) <_ZN68_$LT$core..fmt..builders..PadAdapter$u20$as$u20$core..fmt..Write$GT$9write_str17hc2d1553be630419cE>
   12ee0: f519         	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>
   12ee2: 0189b603     	ld	a2, 0x18(s3)
   12ee6: f7040593     	addi	a1, s0, -0x90
   12eea: 854a         	mv	a0, s2
   12eec: 9602         	jalr	a2
   12eee: f101         	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>
   12ef0: fa843583     	ld	a1, -0x58(s0)
   12ef4: fa043503     	ld	a0, -0x60(s0)
   12ef8: 6d94         	ld	a3, 0x18(a1)

0000000000012efa <.Lpcrel_hi381>:
   12efa: 00003597     	auipc	a1, 0x3
   12efe: fe658593     	addi	a1, a1, -0x1a
   12f02: 4609         	li	a2, 0x2
   12f04: 9682         	jalr	a3
   12f06: a8b1         	j	0x12f62 <.Lpcrel_hi377+0x1c>

0000000000012f08 <.Lpcrel_hi375>:
   12f08: 00003517     	auipc	a0, 0x3
   12f0c: fd350593     	addi	a1, a0, -0x2d
   12f10: 038a3603     	ld	a2, 0x38(s4)
   12f14: 030a3503     	ld	a0, 0x30(s4)
   12f18: 6e18         	ld	a4, 0x18(a2)
   12f1a: 0036c613     	xori	a2, a3, 0x3
   12f1e: 9702         	jalr	a4
   12f20: 4485         	li	s1, 0x1
   12f22: ec0516e3     	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>
   12f26: 8662         	mv	a2, s8
   12f28: 85de         	mv	a1, s7
   12f2a: 038a3683     	ld	a3, 0x38(s4)
   12f2e: 030a3503     	ld	a0, 0x30(s4)
   12f32: 6e94         	ld	a3, 0x18(a3)
   12f34: 9682         	jalr	a3
   12f36: 4485         	li	s1, 0x1
   12f38: ea051be3     	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>
   12f3c: 038a3583     	ld	a1, 0x38(s4)
   12f40: 030a3503     	ld	a0, 0x30(s4)
   12f44: 6d94         	ld	a3, 0x18(a1)

0000000000012f46 <.Lpcrel_hi377>:
   12f46: 00003597     	auipc	a1, 0x3
   12f4a: f3a58593     	addi	a1, a1, -0xc6
   12f4e: 4609         	li	a2, 0x2
   12f50: 9682         	jalr	a3
   12f52: 4485         	li	s1, 0x1
   12f54: e8051de3     	bnez	a0, 0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>
   12f58: 0189b603     	ld	a2, 0x18(s3)
   12f5c: 854a         	mv	a0, s2
   12f5e: 85d2         	mv	a1, s4
   12f60: 9602         	jalr	a2
   12f62: 84aa         	mv	s1, a0
   12f64: b569         	j	0x12dee <_ZN4core3fmt8builders11DebugStruct5field17hea4915c854b53e8aE+0x24>

0000000000012f66 <_ZN4core3fmt5Write9write_fmt17h901a3da00971dfebE>:
   12f66: 1141         	addi	sp, sp, -0x10
   12f68: e406         	sd	ra, 0x8(sp)
   12f6a: e022         	sd	s0, 0x0(sp)
   12f6c: 0800         	addi	s0, sp, 0x10

0000000000012f6e <.Lpcrel_hi547>:
   12f6e: 00003617     	auipc	a2, 0x3
   12f72: f3a60613     	addi	a2, a2, -0xc6
   12f76: 86ae         	mv	a3, a1
   12f78: 85b2         	mv	a1, a2
   12f7a: 8636         	mv	a2, a3
   12f7c: 60a2         	ld	ra, 0x8(sp)
   12f7e: 6402         	ld	s0, 0x0(sp)
   12f80: 0141         	addi	sp, sp, 0x10
   12f82: 00000317     	auipc	t1, 0x0
   12f86: 02830067     	jr	0x28(t1) <_ZN4core3fmt5write17h1b882e4f6891aa5dE>

0000000000012f8a <_ZN59_$LT$core..fmt..Arguments$u20$as$u20$core..fmt..Display$GT$3fmt17hdd88ad703b4b063cE>:
   12f8a: 1141         	addi	sp, sp, -0x10
   12f8c: e406         	sd	ra, 0x8(sp)
   12f8e: e022         	sd	s0, 0x0(sp)
   12f90: 0800         	addi	s0, sp, 0x10
   12f92: 7990         	ld	a2, 0x30(a1)
   12f94: 7d8c         	ld	a1, 0x38(a1)
   12f96: 86aa         	mv	a3, a0
   12f98: 8532         	mv	a0, a2
   12f9a: 8636         	mv	a2, a3
   12f9c: 60a2         	ld	ra, 0x8(sp)
   12f9e: 6402         	ld	s0, 0x0(sp)
   12fa0: 0141         	addi	sp, sp, 0x10
   12fa2: 00000317     	auipc	t1, 0x0
   12fa6: 00830067     	jr	0x8(t1) <_ZN4core3fmt5write17h1b882e4f6891aa5dE>

0000000000012faa <_ZN4core3fmt5write17h1b882e4f6891aa5dE>:
   12faa: 7175         	addi	sp, sp, -0x90
   12fac: e506         	sd	ra, 0x88(sp)
   12fae: e122         	sd	s0, 0x80(sp)
   12fb0: fca6         	sd	s1, 0x78(sp)
   12fb2: f8ca         	sd	s2, 0x70(sp)
   12fb4: f4ce         	sd	s3, 0x68(sp)
   12fb6: f0d2         	sd	s4, 0x60(sp)
   12fb8: ecd6         	sd	s5, 0x58(sp)
   12fba: e8da         	sd	s6, 0x50(sp)
   12fbc: e4de         	sd	s7, 0x48(sp)
   12fbe: e0e2         	sd	s8, 0x40(sp)
   12fc0: 0900         	addi	s0, sp, 0x90
   12fc2: 89b2         	mv	s3, a2
   12fc4: f6043823     	sd	zero, -0x90(s0)
   12fc8: f8043023     	sd	zero, -0x80(s0)
   12fcc: 02000613     	li	a2, 0x20
   12fd0: f8c43823     	sd	a2, -0x70(s0)
   12fd4: 0209b483     	ld	s1, 0x20(s3)
   12fd8: 460d         	li	a2, 0x3
   12fda: f8c40c23     	sb	a2, -0x68(s0)
   12fde: faa43023     	sd	a0, -0x60(s0)
   12fe2: fab43423     	sd	a1, -0x58(s0)
   12fe6: cce1         	beqz	s1, 0x130be <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x114>
   12fe8: 0289b503     	ld	a0, 0x28(s3)
   12fec: 12050463     	beqz	a0, 0x13114 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x16a>
   12ff0: 0009bb03     	ld	s6, 0x0(s3)
   12ff4: 0109ba03     	ld	s4, 0x10(s3)
   12ff8: fff50593     	addi	a1, a0, -0x1
   12ffc: 058e         	slli	a1, a1, 0x3
   12ffe: 818d         	srli	a1, a1, 0x3
   13000: 00158913     	addi	s2, a1, 0x1
   13004: 0b21         	addi	s6, s6, 0x8
   13006: 00351593     	slli	a1, a0, 0x3
   1300a: 051a         	slli	a0, a0, 0x6
   1300c: 40b50ab3     	sub	s5, a0, a1
   13010: 04e1         	addi	s1, s1, 0x18
   13012: 4b89         	li	s7, 0x2
   13014: 4c05         	li	s8, 0x1
   13016: 000b3603     	ld	a2, 0x0(s6)
   1301a: ca19         	beqz	a2, 0x13030 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x86>
   1301c: fa843683     	ld	a3, -0x58(s0)
   13020: fa043503     	ld	a0, -0x60(s0)
   13024: ff8b3583     	ld	a1, -0x8(s6)
   13028: 6e94         	ld	a3, 0x18(a3)
   1302a: 9682         	jalr	a3
   1302c: 10051863     	bnez	a0, 0x1313c <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x192>
   13030: 6888         	ld	a0, 0x10(s1)
   13032: f8a43823     	sd	a0, -0x70(s0)
   13036: 0184c603     	lbu	a2, 0x18(s1)
   1303a: ff84b583     	ld	a1, -0x8(s1)
   1303e: 6088         	ld	a0, 0x0(s1)
   13040: f8c40c23     	sb	a2, -0x68(s0)
   13044: c18d         	beqz	a1, 0x13066 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xbc>
   13046: 01859663     	bne	a1, s8, 0x13052 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xa8>
   1304a: 0512         	slli	a0, a0, 0x4
   1304c: 9552         	add	a0, a0, s4
   1304e: 610c         	ld	a1, 0x0(a0)
   13050: c991         	beqz	a1, 0x13064 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xba>
   13052: fe84b603     	ld	a2, -0x18(s1)
   13056: f6043823     	sd	zero, -0x90(s0)
   1305a: f6a43c23     	sd	a0, -0x88(s0)
   1305e: 01761d63     	bne	a2, s7, 0x13078 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xce>
   13062: a025         	j	0x1308a <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xe0>
   13064: 6508         	ld	a0, 0x8(a0)
   13066: 4585         	li	a1, 0x1
   13068: fe84b603     	ld	a2, -0x18(s1)
   1306c: f6b43823     	sd	a1, -0x90(s0)
   13070: f6a43c23     	sd	a0, -0x88(s0)
   13074: 01760b63     	beq	a2, s7, 0x1308a <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xe0>
   13078: ff04b583     	ld	a1, -0x10(s1)
   1307c: 01861a63     	bne	a2, s8, 0x13090 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xe6>
   13080: 00459513     	slli	a0, a1, 0x4
   13084: 9552         	add	a0, a0, s4
   13086: 610c         	ld	a1, 0x0(a0)
   13088: c199         	beqz	a1, 0x1308e <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xe4>
   1308a: 4601         	li	a2, 0x0
   1308c: a019         	j	0x13092 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0xe8>
   1308e: 650c         	ld	a1, 0x8(a0)
   13090: 4605         	li	a2, 0x1
   13092: 6488         	ld	a0, 0x8(s1)
   13094: 0512         	slli	a0, a0, 0x4
   13096: 00aa06b3     	add	a3, s4, a0
   1309a: 6288         	ld	a0, 0x0(a3)
   1309c: 6694         	ld	a3, 0x8(a3)
   1309e: f8c43023     	sd	a2, -0x80(s0)
   130a2: f8b43423     	sd	a1, -0x78(s0)
   130a6: f7040593     	addi	a1, s0, -0x90
   130aa: 9682         	jalr	a3
   130ac: e941         	bnez	a0, 0x1313c <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x192>
   130ae: 0b41         	addi	s6, s6, 0x10
   130b0: fc8a8a93     	addi	s5, s5, -0x38
   130b4: 03848493     	addi	s1, s1, 0x38
   130b8: f40a9fe3     	bnez	s5, 0x13016 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x6c>
   130bc: a0b9         	j	0x1310a <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x160>
   130be: 0189b503     	ld	a0, 0x18(s3)
   130c2: c929         	beqz	a0, 0x13114 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x16a>
   130c4: 0109b483     	ld	s1, 0x10(s3)
   130c8: 00451a13     	slli	s4, a0, 0x4
   130cc: 9a26         	add	s4, s4, s1
   130ce: 0009ba83     	ld	s5, 0x0(s3)
   130d2: 157d         	addi	a0, a0, -0x1
   130d4: 0512         	slli	a0, a0, 0x4
   130d6: 8111         	srli	a0, a0, 0x4
   130d8: 00150913     	addi	s2, a0, 0x1
   130dc: 0aa1         	addi	s5, s5, 0x8
   130de: 000ab603     	ld	a2, 0x0(s5)
   130e2: ca11         	beqz	a2, 0x130f6 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x14c>
   130e4: fa843683     	ld	a3, -0x58(s0)
   130e8: fa043503     	ld	a0, -0x60(s0)
   130ec: ff8ab583     	ld	a1, -0x8(s5)
   130f0: 6e94         	ld	a3, 0x18(a3)
   130f2: 9682         	jalr	a3
   130f4: e521         	bnez	a0, 0x1313c <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x192>
   130f6: 6088         	ld	a0, 0x0(s1)
   130f8: 6490         	ld	a2, 0x8(s1)
   130fa: f7040593     	addi	a1, s0, -0x90
   130fe: 9602         	jalr	a2
   13100: ed15         	bnez	a0, 0x1313c <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x192>
   13102: 04c1         	addi	s1, s1, 0x10
   13104: 0ac1         	addi	s5, s5, 0x10
   13106: fd449ce3     	bne	s1, s4, 0x130de <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x134>
   1310a: 0089b503     	ld	a0, 0x8(s3)
   1310e: 00a96863     	bltu	s2, a0, 0x1311e <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x174>
   13112: a03d         	j	0x13140 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x196>
   13114: 4901         	li	s2, 0x0
   13116: 0089b503     	ld	a0, 0x8(s3)
   1311a: 02a07363     	bgeu	zero, a0, 0x13140 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x196>
   1311e: 0009b503     	ld	a0, 0x0(s3)
   13122: 0912         	slli	s2, s2, 0x4
   13124: 992a         	add	s2, s2, a0
   13126: fa843683     	ld	a3, -0x58(s0)
   1312a: fa043503     	ld	a0, -0x60(s0)
   1312e: 00093583     	ld	a1, 0x0(s2)
   13132: 00893603     	ld	a2, 0x8(s2)
   13136: 6e94         	ld	a3, 0x18(a3)
   13138: 9682         	jalr	a3
   1313a: c119         	beqz	a0, 0x13140 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x196>
   1313c: 4505         	li	a0, 0x1
   1313e: a011         	j	0x13142 <_ZN4core3fmt5write17h1b882e4f6891aa5dE+0x198>
   13140: 4501         	li	a0, 0x0
   13142: 60aa         	ld	ra, 0x88(sp)
   13144: 640a         	ld	s0, 0x80(sp)
   13146: 74e6         	ld	s1, 0x78(sp)
   13148: 7946         	ld	s2, 0x70(sp)
   1314a: 79a6         	ld	s3, 0x68(sp)
   1314c: 7a06         	ld	s4, 0x60(sp)
   1314e: 6ae6         	ld	s5, 0x58(sp)
   13150: 6b46         	ld	s6, 0x50(sp)
   13152: 6ba6         	ld	s7, 0x48(sp)
   13154: 6c06         	ld	s8, 0x40(sp)
   13156: 6149         	addi	sp, sp, 0x90
   13158: 8082         	ret

000000000001315a <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE>:
   1315a: 7159         	addi	sp, sp, -0x70
   1315c: f486         	sd	ra, 0x68(sp)
   1315e: f0a2         	sd	s0, 0x60(sp)
   13160: eca6         	sd	s1, 0x58(sp)
   13162: e8ca         	sd	s2, 0x50(sp)
   13164: e4ce         	sd	s3, 0x48(sp)
   13166: e0d2         	sd	s4, 0x40(sp)
   13168: fc56         	sd	s5, 0x38(sp)
   1316a: f85a         	sd	s6, 0x30(sp)
   1316c: f45e         	sd	s7, 0x28(sp)
   1316e: f062         	sd	s8, 0x20(sp)
   13170: ec66         	sd	s9, 0x18(sp)
   13172: e86a         	sd	s10, 0x10(sp)
   13174: e46e         	sd	s11, 0x8(sp)
   13176: 1880         	addi	s0, sp, 0x70
   13178: 89be         	mv	s3, a5
   1317a: 893a         	mv	s2, a4
   1317c: 8a36         	mv	s4, a3
   1317e: 8b32         	mv	s6, a2
   13180: c5b9         	beqz	a1, 0x131ce <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x74>
   13182: 02456483     	lwu	s1, 0x24(a0)
   13186: 0014fc13     	andi	s8, s1, 0x1
   1318a: 00110ab7     	lui	s5, 0x110
   1318e: 000c0463     	beqz	s8, 0x13196 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x3c>
   13192: 02b00a93     	li	s5, 0x2b
   13196: 9c4e         	add	s8, s8, s3
   13198: 0044f593     	andi	a1, s1, 0x4
   1319c: c1a9         	beqz	a1, 0x131de <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x84>
   1319e: 02000593     	li	a1, 0x20
   131a2: 04ba7263     	bgeu	s4, a1, 0x131e6 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x8c>
   131a6: 4581         	li	a1, 0x0
   131a8: 000a0f63     	beqz	s4, 0x131c6 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x6c>
   131ac: 014b0633     	add	a2, s6, s4
   131b0: 86da         	mv	a3, s6
   131b2: 00068703     	lb	a4, 0x0(a3)
   131b6: fc072713     	slti	a4, a4, -0x40
   131ba: 00174713     	xori	a4, a4, 0x1
   131be: 0685         	addi	a3, a3, 0x1
   131c0: 95ba         	add	a1, a1, a4
   131c2: fec698e3     	bne	a3, a2, 0x131b2 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x58>
   131c6: 9c2e         	add	s8, s8, a1
   131c8: 610c         	ld	a1, 0x0(a0)
   131ca: c1a5         	beqz	a1, 0x1322a <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xd0>
   131cc: a815         	j	0x13200 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xa6>
   131ce: 5144         	lw	s1, 0x24(a0)
   131d0: 00198c13     	addi	s8, s3, 0x1
   131d4: 02d00a93     	li	s5, 0x2d
   131d8: 0044f593     	andi	a1, s1, 0x4
   131dc: f1e9         	bnez	a1, 0x1319e <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x44>
   131de: 4b01         	li	s6, 0x0
   131e0: 610c         	ld	a1, 0x0(a0)
   131e2: ed99         	bnez	a1, 0x13200 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xa6>
   131e4: a099         	j	0x1322a <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xd0>
   131e6: 8baa         	mv	s7, a0
   131e8: 855a         	mv	a0, s6
   131ea: 85d2         	mv	a1, s4
   131ec: 00001097     	auipc	ra, 0x1
   131f0: 9da080e7     	jalr	-0x626(ra) <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE>
   131f4: 85aa         	mv	a1, a0
   131f6: 855e         	mv	a0, s7
   131f8: 9c2e         	add	s8, s8, a1
   131fa: 000bb583     	ld	a1, 0x0(s7)
   131fe: c595         	beqz	a1, 0x1322a <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xd0>
   13200: 00853c83     	ld	s9, 0x8(a0)
   13204: 039c7363     	bgeu	s8, s9, 0x1322a <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xd0>
   13208: 88a1         	andi	s1, s1, 0x8
   1320a: e0b5         	bnez	s1, 0x1326e <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x114>
   1320c: 02854583     	lbu	a1, 0x28(a0)
   13210: 460d         	li	a2, 0x3
   13212: 00c59363     	bne	a1, a2, 0x13218 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0xbe>
   13216: 4585         	li	a1, 0x1
   13218: 418c8cb3     	sub	s9, s9, s8
   1321c: c5d5         	beqz	a1, 0x132c8 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x16e>
   1321e: 4605         	li	a2, 0x1
   13220: 08c59f63     	bne	a1, a2, 0x132be <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x164>
   13224: 85e6         	mv	a1, s9
   13226: 4c81         	li	s9, 0x0
   13228: a045         	j	0x132c8 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x16e>
   1322a: 03053b83     	ld	s7, 0x30(a0)
   1322e: 7d04         	ld	s1, 0x38(a0)
   13230: 855e         	mv	a0, s7
   13232: 85a6         	mv	a1, s1
   13234: 8656         	mv	a2, s5
   13236: 86da         	mv	a3, s6
   13238: 8752         	mv	a4, s4
   1323a: 00000097     	auipc	ra, 0x0
   1323e: 144080e7     	jalr	0x144(ra) <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE>
   13242: 85aa         	mv	a1, a0
   13244: 4505         	li	a0, 0x1
   13246: e1d5         	bnez	a1, 0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>
   13248: 6c9c         	ld	a5, 0x18(s1)
   1324a: 855e         	mv	a0, s7
   1324c: 85ca         	mv	a1, s2
   1324e: 864e         	mv	a2, s3
   13250: 70a6         	ld	ra, 0x68(sp)
   13252: 7406         	ld	s0, 0x60(sp)
   13254: 64e6         	ld	s1, 0x58(sp)
   13256: 6946         	ld	s2, 0x50(sp)
   13258: 69a6         	ld	s3, 0x48(sp)
   1325a: 6a06         	ld	s4, 0x40(sp)
   1325c: 7ae2         	ld	s5, 0x38(sp)
   1325e: 7b42         	ld	s6, 0x30(sp)
   13260: 7ba2         	ld	s7, 0x28(sp)
   13262: 7c02         	ld	s8, 0x20(sp)
   13264: 6ce2         	ld	s9, 0x18(sp)
   13266: 6d42         	ld	s10, 0x10(sp)
   13268: 6da2         	ld	s11, 0x8(sp)
   1326a: 6165         	addi	sp, sp, 0x70
   1326c: 8782         	jr	a5
   1326e: 5104         	lw	s1, 0x20(a0)
   13270: 03000593     	li	a1, 0x30
   13274: 02854603     	lbu	a2, 0x28(a0)
   13278: f8c43823     	sd	a2, -0x70(s0)
   1327c: 03053b83     	ld	s7, 0x30(a0)
   13280: 03853d03     	ld	s10, 0x38(a0)
   13284: d10c         	sw	a1, 0x20(a0)
   13286: 4585         	li	a1, 0x1
   13288: 8daa         	mv	s11, a0
   1328a: 02b50423     	sb	a1, 0x28(a0)
   1328e: 855e         	mv	a0, s7
   13290: 85ea         	mv	a1, s10
   13292: 8656         	mv	a2, s5
   13294: 86da         	mv	a3, s6
   13296: 8752         	mv	a4, s4
   13298: 00000097     	auipc	ra, 0x0
   1329c: 0e6080e7     	jalr	0xe6(ra) <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE>
   132a0: e521         	bnez	a0, 0x132e8 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x18e>
   132a2: 8a26         	mv	s4, s1
   132a4: 418c84b3     	sub	s1, s9, s8
   132a8: 0485         	addi	s1, s1, 0x1
   132aa: 14fd         	addi	s1, s1, -0x1
   132ac: c4cd         	beqz	s1, 0x13356 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x1fc>
   132ae: 020d3603     	ld	a2, 0x20(s10)
   132b2: 03000593     	li	a1, 0x30
   132b6: 855e         	mv	a0, s7
   132b8: 9602         	jalr	a2
   132ba: d965         	beqz	a0, 0x132aa <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x150>
   132bc: a035         	j	0x132e8 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x18e>
   132be: 001cd593     	srli	a1, s9, 0x1
   132c2: 0c85         	addi	s9, s9, 0x1
   132c4: 001cdc93     	srli	s9, s9, 0x1
   132c8: 03053b83     	ld	s7, 0x30(a0)
   132cc: 03853d03     	ld	s10, 0x38(a0)
   132d0: 02052c03     	lw	s8, 0x20(a0)
   132d4: 00158493     	addi	s1, a1, 0x1
   132d8: 14fd         	addi	s1, s1, -0x1
   132da: c49d         	beqz	s1, 0x13308 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x1ae>
   132dc: 020d3603     	ld	a2, 0x20(s10)
   132e0: 855e         	mv	a0, s7
   132e2: 85e2         	mv	a1, s8
   132e4: 9602         	jalr	a2
   132e6: d96d         	beqz	a0, 0x132d8 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x17e>
   132e8: 4505         	li	a0, 0x1
   132ea: 70a6         	ld	ra, 0x68(sp)
   132ec: 7406         	ld	s0, 0x60(sp)
   132ee: 64e6         	ld	s1, 0x58(sp)
   132f0: 6946         	ld	s2, 0x50(sp)
   132f2: 69a6         	ld	s3, 0x48(sp)
   132f4: 6a06         	ld	s4, 0x40(sp)
   132f6: 7ae2         	ld	s5, 0x38(sp)
   132f8: 7b42         	ld	s6, 0x30(sp)
   132fa: 7ba2         	ld	s7, 0x28(sp)
   132fc: 7c02         	ld	s8, 0x20(sp)
   132fe: 6ce2         	ld	s9, 0x18(sp)
   13300: 6d42         	ld	s10, 0x10(sp)
   13302: 6da2         	ld	s11, 0x8(sp)
   13304: 6165         	addi	sp, sp, 0x70
   13306: 8082         	ret
   13308: 855e         	mv	a0, s7
   1330a: 85ea         	mv	a1, s10
   1330c: 8656         	mv	a2, s5
   1330e: 86da         	mv	a3, s6
   13310: 8752         	mv	a4, s4
   13312: 00000097     	auipc	ra, 0x0
   13316: 06c080e7     	jalr	0x6c(ra) <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE>
   1331a: 85aa         	mv	a1, a0
   1331c: 4505         	li	a0, 0x1
   1331e: f5f1         	bnez	a1, 0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>
   13320: 018d3683     	ld	a3, 0x18(s10)
   13324: 855e         	mv	a0, s7
   13326: 85ca         	mv	a1, s2
   13328: 864e         	mv	a2, s3
   1332a: 9682         	jalr	a3
   1332c: 85aa         	mv	a1, a0
   1332e: 4505         	li	a0, 0x1
   13330: fdcd         	bnez	a1, 0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>
   13332: 41900933     	neg	s2, s9
   13336: 59fd         	li	s3, -0x1
   13338: 54fd         	li	s1, -0x1
   1333a: 00990533     	add	a0, s2, s1
   1333e: 03350d63     	beq	a0, s3, 0x13378 <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x21e>
   13342: 020d3603     	ld	a2, 0x20(s10)
   13346: 855e         	mv	a0, s7
   13348: 85e2         	mv	a1, s8
   1334a: 9602         	jalr	a2
   1334c: 0485         	addi	s1, s1, 0x1
   1334e: d575         	beqz	a0, 0x1333a <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x1e0>
   13350: 0194b533     	sltu	a0, s1, s9
   13354: bf59         	j	0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>
   13356: 018d3683     	ld	a3, 0x18(s10)
   1335a: 855e         	mv	a0, s7
   1335c: 85ca         	mv	a1, s2
   1335e: 864e         	mv	a2, s3
   13360: 9682         	jalr	a3
   13362: 85aa         	mv	a1, a0
   13364: 4505         	li	a0, 0x1
   13366: f1d1         	bnez	a1, 0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>
   13368: 4501         	li	a0, 0x0
   1336a: 034da023     	sw	s4, 0x20(s11)
   1336e: f9043583     	ld	a1, -0x70(s0)
   13372: 02bd8423     	sb	a1, 0x28(s11)
   13376: bf95         	j	0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>
   13378: 019cb533     	sltu	a0, s9, s9
   1337c: b7bd         	j	0x132ea <_ZN4core3fmt9Formatter12pad_integral17hf929a3c1d8b1493cE+0x190>

000000000001337e <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE>:
   1337e: 7179         	addi	sp, sp, -0x30
   13380: f406         	sd	ra, 0x28(sp)
   13382: f022         	sd	s0, 0x20(sp)
   13384: ec26         	sd	s1, 0x18(sp)
   13386: e84a         	sd	s2, 0x10(sp)
   13388: e44e         	sd	s3, 0x8(sp)
   1338a: e052         	sd	s4, 0x0(sp)
   1338c: 1800         	addi	s0, sp, 0x30
   1338e: 001107b7     	lui	a5, 0x110
   13392: 893a         	mv	s2, a4
   13394: 8a36         	mv	s4, a3
   13396: 89ae         	mv	s3, a1
   13398: 00f60b63     	beq	a2, a5, 0x133ae <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE+0x30>
   1339c: 0209b683     	ld	a3, 0x20(s3)
   133a0: 84aa         	mv	s1, a0
   133a2: 85b2         	mv	a1, a2
   133a4: 9682         	jalr	a3
   133a6: 862a         	mv	a2, a0
   133a8: 8526         	mv	a0, s1
   133aa: 4585         	li	a1, 0x1
   133ac: e205         	bnez	a2, 0x133cc <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE+0x4e>
   133ae: 000a0e63     	beqz	s4, 0x133ca <_ZN4core3fmt9Formatter12pad_integral12write_prefix17h5c73c21e7ab6ccbdE+0x4c>
   133b2: 0189b783     	ld	a5, 0x18(s3)
   133b6: 85d2         	mv	a1, s4
   133b8: 864a         	mv	a2, s2
   133ba: 70a2         	ld	ra, 0x28(sp)
   133bc: 7402         	ld	s0, 0x20(sp)
   133be: 64e2         	ld	s1, 0x18(sp)
   133c0: 6942         	ld	s2, 0x10(sp)
   133c2: 69a2         	ld	s3, 0x8(sp)
   133c4: 6a02         	ld	s4, 0x0(sp)
   133c6: 6145         	addi	sp, sp, 0x30
   133c8: 8782         	jr	a5
   133ca: 4581         	li	a1, 0x0
   133cc: 852e         	mv	a0, a1
   133ce: 70a2         	ld	ra, 0x28(sp)
   133d0: 7402         	ld	s0, 0x20(sp)
   133d2: 64e2         	ld	s1, 0x18(sp)
   133d4: 6942         	ld	s2, 0x10(sp)
   133d6: 69a2         	ld	s3, 0x8(sp)
   133d8: 6a02         	ld	s4, 0x0(sp)
   133da: 6145         	addi	sp, sp, 0x30
   133dc: 8082         	ret

00000000000133de <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE>:
   133de: 715d         	addi	sp, sp, -0x50
   133e0: e486         	sd	ra, 0x48(sp)
   133e2: e0a2         	sd	s0, 0x40(sp)
   133e4: fc26         	sd	s1, 0x38(sp)
   133e6: f84a         	sd	s2, 0x30(sp)
   133e8: f44e         	sd	s3, 0x28(sp)
   133ea: f052         	sd	s4, 0x20(sp)
   133ec: ec56         	sd	s5, 0x18(sp)
   133ee: e85a         	sd	s6, 0x10(sp)
   133f0: e45e         	sd	s7, 0x8(sp)
   133f2: 0880         	addi	s0, sp, 0x50
   133f4: 6114         	ld	a3, 0x0(a0)
   133f6: 6918         	ld	a4, 0x10(a0)
   133f8: 00e6e7b3     	or	a5, a3, a4
   133fc: 8932         	mv	s2, a2
   133fe: 89ae         	mv	s3, a1
   13400: 10078163     	beqz	a5, 0x13502 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x124>
   13404: 8b05         	andi	a4, a4, 0x1
   13406: cb51         	beqz	a4, 0x1349a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xbc>
   13408: 6d18         	ld	a4, 0x18(a0)
   1340a: 01298633     	add	a2, s3, s2
   1340e: 4581         	li	a1, 0x0
   13410: c721         	beqz	a4, 0x13458 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x7a>
   13412: 0e000893     	li	a7, 0xe0
   13416: 0f000813     	li	a6, 0xf0
   1341a: 84ce         	mv	s1, s3
   1341c: a809         	j	0x1342e <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x50>
   1341e: 00148793     	addi	a5, s1, 0x1
   13422: 8c8d         	sub	s1, s1, a1
   13424: 177d         	addi	a4, a4, -0x1
   13426: 409785b3     	sub	a1, a5, s1
   1342a: 84be         	mv	s1, a5
   1342c: c71d         	beqz	a4, 0x1345a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x7c>
   1342e: 06c48663     	beq	s1, a2, 0x1349a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xbc>
   13432: 00048783     	lb	a5, 0x0(s1)
   13436: fe07d4e3     	bgez	a5, 0x1341e <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x40>
   1343a: 0ff7f793     	andi	a5, a5, 0xff
   1343e: 0117e763     	bltu	a5, a7, 0x1344c <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x6e>
   13442: 0107e863     	bltu	a5, a6, 0x13452 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x74>
   13446: 00448793     	addi	a5, s1, 0x4
   1344a: bfe1         	j	0x13422 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x44>
   1344c: 00248793     	addi	a5, s1, 0x2
   13450: bfc9         	j	0x13422 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x44>
   13452: 00348793     	addi	a5, s1, 0x3
   13456: b7f1         	j	0x13422 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x44>
   13458: 87ce         	mv	a5, s3
   1345a: 04c78063     	beq	a5, a2, 0x1349a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xbc>
   1345e: 00078603     	lb	a2, 0x0(a5)
   13462: 00065663     	bgez	a2, 0x1346e <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x90>
   13466: 0ff67613     	andi	a2, a2, 0xff
   1346a: 0e000713     	li	a4, 0xe0
   1346e: c18d         	beqz	a1, 0x13490 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xb2>
   13470: 0125fe63     	bgeu	a1, s2, 0x1348c <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xae>
   13474: 00b98633     	add	a2, s3, a1
   13478: 00060603     	lb	a2, 0x0(a2)
   1347c: fc000713     	li	a4, -0x40
   13480: 00e65863     	bge	a2, a4, 0x13490 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xb2>
   13484: 4601         	li	a2, 0x0
   13486: 00001863     	bnez	zero, 0x13496 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xb8>
   1348a: a801         	j	0x1349a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xbc>
   1348c: ff259ce3     	bne	a1, s2, 0x13484 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xa6>
   13490: 864e         	mv	a2, s3
   13492: 00098463     	beqz	s3, 0x1349a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xbc>
   13496: 892e         	mv	s2, a1
   13498: 89b2         	mv	s3, a2
   1349a: c6a5         	beqz	a3, 0x13502 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x124>
   1349c: 6504         	ld	s1, 0x8(a0)
   1349e: 02000593     	li	a1, 0x20
   134a2: 04b97563     	bgeu	s2, a1, 0x134ec <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x10e>
   134a6: 4581         	li	a1, 0x0
   134a8: 00090f63     	beqz	s2, 0x134c6 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xe8>
   134ac: 01298633     	add	a2, s3, s2
   134b0: 86ce         	mv	a3, s3
   134b2: 00068703     	lb	a4, 0x0(a3)
   134b6: fc072713     	slti	a4, a4, -0x40
   134ba: 00174713     	xori	a4, a4, 0x1
   134be: 0685         	addi	a3, a3, 0x1
   134c0: 95ba         	add	a1, a1, a4
   134c2: fec698e3     	bne	a3, a2, 0x134b2 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xd4>
   134c6: 0295fe63     	bgeu	a1, s1, 0x13502 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x124>
   134ca: 02854603     	lbu	a2, 0x28(a0)
   134ce: ffd60693     	addi	a3, a2, -0x3
   134d2: 0016b693     	seqz	a3, a3
   134d6: 16fd         	addi	a3, a3, -0x1
   134d8: 8e75         	and	a2, a2, a3
   134da: 40b48ab3     	sub	s5, s1, a1
   134de: c639         	beqz	a2, 0x1352c <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x14e>
   134e0: 4585         	li	a1, 0x1
   134e2: 04b61063     	bne	a2, a1, 0x13522 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x144>
   134e6: 8656         	mv	a2, s5
   134e8: 4a81         	li	s5, 0x0
   134ea: a089         	j	0x1352c <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x14e>
   134ec: 8a2a         	mv	s4, a0
   134ee: 854e         	mv	a0, s3
   134f0: 85ca         	mv	a1, s2
   134f2: 00000097     	auipc	ra, 0x0
   134f6: 6d4080e7     	jalr	0x6d4(ra) <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE>
   134fa: 85aa         	mv	a1, a0
   134fc: 8552         	mv	a0, s4
   134fe: fc95e6e3     	bltu	a1, s1, 0x134ca <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0xec>
   13502: 7d0c         	ld	a1, 0x38(a0)
   13504: 7908         	ld	a0, 0x30(a0)
   13506: 6d9c         	ld	a5, 0x18(a1)
   13508: 85ce         	mv	a1, s3
   1350a: 864a         	mv	a2, s2
   1350c: 60a6         	ld	ra, 0x48(sp)
   1350e: 6406         	ld	s0, 0x40(sp)
   13510: 74e2         	ld	s1, 0x38(sp)
   13512: 7942         	ld	s2, 0x30(sp)
   13514: 79a2         	ld	s3, 0x28(sp)
   13516: 7a02         	ld	s4, 0x20(sp)
   13518: 6ae2         	ld	s5, 0x18(sp)
   1351a: 6b42         	ld	s6, 0x10(sp)
   1351c: 6ba2         	ld	s7, 0x8(sp)
   1351e: 6161         	addi	sp, sp, 0x50
   13520: 8782         	jr	a5
   13522: 001ad613     	srli	a2, s5, 0x1
   13526: 0a85         	addi	s5, s5, 0x1
   13528: 001ada93     	srli	s5, s5, 0x1
   1352c: 03053a03     	ld	s4, 0x30(a0)
   13530: 03853b83     	ld	s7, 0x38(a0)
   13534: 02052b03     	lw	s6, 0x20(a0)
   13538: 00160493     	addi	s1, a2, 0x1
   1353c: 14fd         	addi	s1, s1, -0x1
   1353e: c889         	beqz	s1, 0x13550 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x172>
   13540: 020bb603     	ld	a2, 0x20(s7)
   13544: 8552         	mv	a0, s4
   13546: 85da         	mv	a1, s6
   13548: 9602         	jalr	a2
   1354a: d96d         	beqz	a0, 0x1353c <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x15e>
   1354c: 4505         	li	a0, 0x1
   1354e: a82d         	j	0x13588 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x1aa>
   13550: 018bb683     	ld	a3, 0x18(s7)
   13554: 8552         	mv	a0, s4
   13556: 85ce         	mv	a1, s3
   13558: 864a         	mv	a2, s2
   1355a: 9682         	jalr	a3
   1355c: 85aa         	mv	a1, a0
   1355e: 4505         	li	a0, 0x1
   13560: e585         	bnez	a1, 0x13588 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x1aa>
   13562: 41500933     	neg	s2, s5
   13566: 59fd         	li	s3, -0x1
   13568: 54fd         	li	s1, -0x1
   1356a: 00990533     	add	a0, s2, s1
   1356e: 01350a63     	beq	a0, s3, 0x13582 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x1a4>
   13572: 020bb603     	ld	a2, 0x20(s7)
   13576: 8552         	mv	a0, s4
   13578: 85da         	mv	a1, s6
   1357a: 9602         	jalr	a2
   1357c: 0485         	addi	s1, s1, 0x1
   1357e: d575         	beqz	a0, 0x1356a <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x18c>
   13580: a011         	j	0x13584 <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE+0x1a6>
   13582: 84d6         	mv	s1, s5
   13584: 0154b533     	sltu	a0, s1, s5
   13588: 60a6         	ld	ra, 0x48(sp)
   1358a: 6406         	ld	s0, 0x40(sp)
   1358c: 74e2         	ld	s1, 0x38(sp)
   1358e: 7942         	ld	s2, 0x30(sp)
   13590: 79a2         	ld	s3, 0x28(sp)
   13592: 7a02         	ld	s4, 0x20(sp)
   13594: 6ae2         	ld	s5, 0x18(sp)
   13596: 6b42         	ld	s6, 0x10(sp)
   13598: 6ba2         	ld	s7, 0x8(sp)
   1359a: 6161         	addi	sp, sp, 0x50
   1359c: 8082         	ret

000000000001359e <_ZN57_$LT$core..fmt..Formatter$u20$as$u20$core..fmt..Write$GT$9write_str17h41d63e4bf0c652c7E>:
   1359e: 1141         	addi	sp, sp, -0x10
   135a0: e406         	sd	ra, 0x8(sp)
   135a2: e022         	sd	s0, 0x0(sp)
   135a4: 0800         	addi	s0, sp, 0x10
   135a6: 7d14         	ld	a3, 0x38(a0)
   135a8: 7908         	ld	a0, 0x30(a0)
   135aa: 6e9c         	ld	a5, 0x18(a3)
   135ac: 60a2         	ld	ra, 0x8(sp)
   135ae: 6402         	ld	s0, 0x0(sp)
   135b0: 0141         	addi	sp, sp, 0x10
   135b2: 8782         	jr	a5

00000000000135b4 <_ZN4core3fmt9Formatter26debug_struct_field2_finish17hb31ed29359c11ddfE>:
   135b4: 7159         	addi	sp, sp, -0x70
   135b6: f486         	sd	ra, 0x68(sp)
   135b8: f0a2         	sd	s0, 0x60(sp)
   135ba: eca6         	sd	s1, 0x58(sp)
   135bc: e8ca         	sd	s2, 0x50(sp)
   135be: e4ce         	sd	s3, 0x48(sp)
   135c0: e0d2         	sd	s4, 0x40(sp)
   135c2: fc56         	sd	s5, 0x38(sp)
   135c4: f85a         	sd	s6, 0x30(sp)
   135c6: f45e         	sd	s7, 0x28(sp)
   135c8: f062         	sd	s8, 0x20(sp)
   135ca: ec66         	sd	s9, 0x18(sp)
   135cc: 1880         	addi	s0, sp, 0x70
   135ce: 84aa         	mv	s1, a0
   135d0: 00043903     	ld	s2, 0x0(s0)
   135d4: 03853283     	ld	t0, 0x38(a0)
   135d8: 00843983     	ld	s3, 0x8(s0)
   135dc: 01043a03     	ld	s4, 0x10(s0)
   135e0: 7908         	ld	a0, 0x30(a0)
   135e2: 0182b303     	ld	t1, 0x18(t0)
   135e6: 8ac6         	mv	s5, a7
   135e8: 8b42         	mv	s6, a6
   135ea: 8bbe         	mv	s7, a5
   135ec: 8c3a         	mv	s8, a4
   135ee: 8cb6         	mv	s9, a3
   135f0: 9302         	jalr	t1
   135f2: f8943c23     	sd	s1, -0x68(s0)
   135f6: faa40023     	sb	a0, -0x60(s0)
   135fa: fa0400a3     	sb	zero, -0x5f(s0)
   135fe: f9840513     	addi	a0, s0, -0x68
   13602: 85e6         	mv	a1, s9
   13604: 8662         	mv	a2, s8
   13606: 86de         	mv	a3, s7
   13608: 875a         	mv	a4, s6
   1360a: fffff097     	auipc	ra, 0xfffff
   1360e: 7c0080e7     	jalr	0x7c0(ra) <__rust_no_alloc_shim_is_unstable+0xffff3ca1>
   13612: f9840513     	addi	a0, s0, -0x68
   13616: 85d6         	mv	a1, s5
   13618: 864a         	mv	a2, s2
   1361a: 86ce         	mv	a3, s3
   1361c: 8752         	mv	a4, s4
   1361e: fffff097     	auipc	ra, 0xfffff
   13622: 7ac080e7     	jalr	0x7ac(ra) <__rust_no_alloc_shim_is_unstable+0xffff3ca1>
   13626: fa144603     	lbu	a2, -0x5f(s0)
   1362a: fa044583     	lbu	a1, -0x60(s0)
   1362e: 00b66533     	or	a0, a2, a1
   13632: ca1d         	beqz	a2, 0x13668 <.Lpcrel_hi558+0xc>
   13634: 8985         	andi	a1, a1, 0x1
   13636: e98d         	bnez	a1, 0x13668 <.Lpcrel_hi558+0xc>
   13638: f9843503     	ld	a0, -0x68(s0)
   1363c: 02454583     	lbu	a1, 0x24(a0)
   13640: 8991         	andi	a1, a1, 0x4
   13642: e991         	bnez	a1, 0x13656 <.Lpcrel_hi557+0xc>
   13644: 7d0c         	ld	a1, 0x38(a0)
   13646: 7908         	ld	a0, 0x30(a0)
   13648: 6d94         	ld	a3, 0x18(a1)

000000000001364a <.Lpcrel_hi557>:
   1364a: 00003597     	auipc	a1, 0x3
   1364e: 89958593     	addi	a1, a1, -0x767
   13652: 4609         	li	a2, 0x2
   13654: a809         	j	0x13666 <.Lpcrel_hi558+0xa>
   13656: 7d0c         	ld	a1, 0x38(a0)
   13658: 7908         	ld	a0, 0x30(a0)
   1365a: 6d94         	ld	a3, 0x18(a1)

000000000001365c <.Lpcrel_hi558>:
   1365c: 00003597     	auipc	a1, 0x3
   13660: 88658593     	addi	a1, a1, -0x77a
   13664: 4605         	li	a2, 0x1
   13666: 9682         	jalr	a3
   13668: 8905         	andi	a0, a0, 0x1
   1366a: 70a6         	ld	ra, 0x68(sp)
   1366c: 7406         	ld	s0, 0x60(sp)
   1366e: 64e6         	ld	s1, 0x58(sp)
   13670: 6946         	ld	s2, 0x50(sp)
   13672: 69a6         	ld	s3, 0x48(sp)
   13674: 6a06         	ld	s4, 0x40(sp)
   13676: 7ae2         	ld	s5, 0x38(sp)
   13678: 7b42         	ld	s6, 0x30(sp)
   1367a: 7ba2         	ld	s7, 0x28(sp)
   1367c: 7c02         	ld	s8, 0x20(sp)
   1367e: 6ce2         	ld	s9, 0x18(sp)
   13680: 6165         	addi	sp, sp, 0x70
   13682: 8082         	ret

0000000000013684 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE>:
   13684: 7171         	addi	sp, sp, -0xb0
   13686: f506         	sd	ra, 0xa8(sp)
   13688: f122         	sd	s0, 0xa0(sp)
   1368a: ed26         	sd	s1, 0x98(sp)
   1368c: e94a         	sd	s2, 0x90(sp)
   1368e: e54e         	sd	s3, 0x88(sp)
   13690: e152         	sd	s4, 0x80(sp)
   13692: fcd6         	sd	s5, 0x78(sp)
   13694: f8da         	sd	s6, 0x70(sp)
   13696: f4de         	sd	s7, 0x68(sp)
   13698: f0e2         	sd	s8, 0x60(sp)
   1369a: 1900         	addi	s0, sp, 0xb0
   1369c: 8b2a         	mv	s6, a0
   1369e: 03853b83     	ld	s7, 0x38(a0)
   136a2: 03053c03     	ld	s8, 0x30(a0)
   136a6: 018bb483     	ld	s1, 0x18(s7)
   136aa: 8aba         	mv	s5, a4
   136ac: 8a36         	mv	s4, a3
   136ae: 8932         	mv	s2, a2
   136b0: 8562         	mv	a0, s8
   136b2: 9482         	jalr	s1
   136b4: 4985         	li	s3, 0x1
   136b6: cd11         	beqz	a0, 0x136d2 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x4e>
   136b8: 854e         	mv	a0, s3
   136ba: 70aa         	ld	ra, 0xa8(sp)
   136bc: 740a         	ld	s0, 0xa0(sp)
   136be: 64ea         	ld	s1, 0x98(sp)
   136c0: 694a         	ld	s2, 0x90(sp)
   136c2: 69aa         	ld	s3, 0x88(sp)
   136c4: 6a0a         	ld	s4, 0x80(sp)
   136c6: 7ae6         	ld	s5, 0x78(sp)
   136c8: 7b46         	ld	s6, 0x70(sp)
   136ca: 7ba6         	ld	s7, 0x68(sp)
   136cc: 7c06         	ld	s8, 0x60(sp)
   136ce: 614d         	addi	sp, sp, 0xb0
   136d0: 8082         	ret
   136d2: 024b4503     	lbu	a0, 0x24(s6)
   136d6: 8911         	andi	a0, a0, 0x4
   136d8: e10d         	bnez	a0, 0x136fa <.Lpcrel_hi570>

00000000000136da <.Lpcrel_hi569>:
   136da: 00003517     	auipc	a0, 0x3
   136de: 80b50593     	addi	a1, a0, -0x7f5
   136e2: 4605         	li	a2, 0x1
   136e4: 4985         	li	s3, 0x1
   136e6: 8562         	mv	a0, s8
   136e8: 9482         	jalr	s1
   136ea: f579         	bnez	a0, 0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>
   136ec: 018ab603     	ld	a2, 0x18(s5)
   136f0: 8552         	mv	a0, s4
   136f2: 85da         	mv	a1, s6
   136f4: 9602         	jalr	a2
   136f6: f169         	bnez	a0, 0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>
   136f8: a849         	j	0x1378a <.Lpcrel_hi572+0xe>

00000000000136fa <.Lpcrel_hi570>:
   136fa: 00002517     	auipc	a0, 0x2
   136fe: 7ec50593     	addi	a1, a0, 0x7ec
   13702: 4609         	li	a2, 0x2
   13704: 8562         	mv	a0, s8
   13706: 9482         	jalr	s1
   13708: f945         	bnez	a0, 0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>
   1370a: f5843823     	sd	s8, -0xb0(s0)
   1370e: f5743c23     	sd	s7, -0xa8(s0)
   13712: f6f40513     	addi	a0, s0, -0x91
   13716: f6a43023     	sd	a0, -0xa0(s0)
   1371a: 000b3503     	ld	a0, 0x0(s6)
   1371e: 008b3583     	ld	a1, 0x8(s6)
   13722: 010b3603     	ld	a2, 0x10(s6)
   13726: 018b3683     	ld	a3, 0x18(s6)
   1372a: f6a43823     	sd	a0, -0x90(s0)
   1372e: f6b43c23     	sd	a1, -0x88(s0)
   13732: f8c43023     	sd	a2, -0x80(s0)
   13736: f8d43423     	sd	a3, -0x78(s0)
   1373a: 020b3503     	ld	a0, 0x20(s6)
   1373e: 028b3583     	ld	a1, 0x28(s6)
   13742: 4985         	li	s3, 0x1
   13744: f73407a3     	sb	s3, -0x91(s0)
   13748: f8a43823     	sd	a0, -0x70(s0)
   1374c: f8b43c23     	sd	a1, -0x68(s0)
   13750: f5040513     	addi	a0, s0, -0xb0
   13754: faa43023     	sd	a0, -0x60(s0)
   13758: 018ab603     	ld	a2, 0x18(s5)

000000000001375c <.Lpcrel_hi571>:
   1375c: 00002517     	auipc	a0, 0x2
   13760: 74c50513     	addi	a0, a0, 0x74c
   13764: faa43423     	sd	a0, -0x58(s0)
   13768: f7040593     	addi	a1, s0, -0x90
   1376c: 8552         	mv	a0, s4
   1376e: 9602         	jalr	a2
   13770: f521         	bnez	a0, 0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>
   13772: fa843583     	ld	a1, -0x58(s0)
   13776: fa043503     	ld	a0, -0x60(s0)
   1377a: 6d94         	ld	a3, 0x18(a1)

000000000001377c <.Lpcrel_hi572>:
   1377c: 00002597     	auipc	a1, 0x2
   13780: 76458593     	addi	a1, a1, 0x764
   13784: 4609         	li	a2, 0x2
   13786: 9682         	jalr	a3
   13788: f905         	bnez	a0, 0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>
   1378a: 02091363     	bnez	s2, 0x137b0 <.Lpcrel_hi574+0x10>
   1378e: 024b4503     	lbu	a0, 0x24(s6)
   13792: 8911         	andi	a0, a0, 0x4
   13794: ed11         	bnez	a0, 0x137b0 <.Lpcrel_hi574+0x10>
   13796: 038b3583     	ld	a1, 0x38(s6)
   1379a: 030b3503     	ld	a0, 0x30(s6)
   1379e: 6d94         	ld	a3, 0x18(a1)

00000000000137a0 <.Lpcrel_hi574>:
   137a0: 00002597     	auipc	a1, 0x2
   137a4: 74858593     	addi	a1, a1, 0x748
   137a8: 4605         	li	a2, 0x1
   137aa: 4985         	li	s3, 0x1
   137ac: 9682         	jalr	a3
   137ae: f509         	bnez	a0, 0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>
   137b0: 038b3583     	ld	a1, 0x38(s6)
   137b4: 030b3503     	ld	a0, 0x30(s6)
   137b8: 6d94         	ld	a3, 0x18(a1)

00000000000137ba <.Lpcrel_hi573>:
   137ba: 00002597     	auipc	a1, 0x2
   137be: 58558593     	addi	a1, a1, 0x585
   137c2: 4605         	li	a2, 0x1
   137c4: 9682         	jalr	a3
   137c6: 89aa         	mv	s3, a0
   137c8: bdc5         	j	0x136b8 <_ZN4core3fmt9Formatter25debug_tuple_field1_finish17ha02a23fe25eaf66cE+0x34>

00000000000137ca <_ZN42_$LT$str$u20$as$u20$core..fmt..Display$GT$3fmt17h6c34711bbfb2649bE>:
   137ca: 1141         	addi	sp, sp, -0x10
   137cc: e406         	sd	ra, 0x8(sp)
   137ce: e022         	sd	s0, 0x0(sp)
   137d0: 0800         	addi	s0, sp, 0x10
   137d2: 86ae         	mv	a3, a1
   137d4: 85aa         	mv	a1, a0
   137d6: 8532         	mv	a0, a2
   137d8: 8636         	mv	a2, a3
   137da: 60a2         	ld	ra, 0x8(sp)
   137dc: 6402         	ld	s0, 0x0(sp)
   137de: 0141         	addi	sp, sp, 0x10
   137e0: 00000317     	auipc	t1, 0x0
   137e4: bfe30067     	jr	-0x402(t1) <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE>

00000000000137e8 <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E>:
   137e8: 86ae         	mv	a3, a1
   137ea: 6190         	ld	a2, 0x0(a1)
   137ec: 6998         	ld	a4, 0x10(a1)
   137ee: 410c         	lw	a1, 0x0(a0)
   137f0: 8e59         	or	a2, a2, a4
   137f2: e609         	bnez	a2, 0x137fc <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0x14>
   137f4: 7e90         	ld	a2, 0x38(a3)
   137f6: 7a88         	ld	a0, 0x30(a3)
   137f8: 721c         	ld	a5, 0x20(a2)
   137fa: 8782         	jr	a5
   137fc: 1101         	addi	sp, sp, -0x20
   137fe: ec06         	sd	ra, 0x18(sp)
   13800: e822         	sd	s0, 0x10(sp)
   13802: 1000         	addi	s0, sp, 0x20
   13804: 08000513     	li	a0, 0x80
   13808: fe042623     	sw	zero, -0x14(s0)
   1380c: 00a5f663     	bgeu	a1, a0, 0x13818 <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0x30>
   13810: feb40623     	sb	a1, -0x14(s0)
   13814: 4605         	li	a2, 0x1
   13816: a069         	j	0x138a0 <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0xb8>
   13818: 00b5d51b     	srliw	a0, a1, 0xb
   1381c: ed19         	bnez	a0, 0x1383a <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0x52>
   1381e: 0065d513     	srli	a0, a1, 0x6
   13822: 0c056513     	ori	a0, a0, 0xc0
   13826: fea40623     	sb	a0, -0x14(s0)
   1382a: 03f5f513     	andi	a0, a1, 0x3f
   1382e: 08050513     	addi	a0, a0, 0x80
   13832: fea406a3     	sb	a0, -0x13(s0)
   13836: 4609         	li	a2, 0x2
   13838: a0a5         	j	0x138a0 <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0xb8>
   1383a: 0105d51b     	srliw	a0, a1, 0x10
   1383e: e515         	bnez	a0, 0x1386a <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0x82>
   13840: 00c5d513     	srli	a0, a1, 0xc
   13844: 0e056513     	ori	a0, a0, 0xe0
   13848: fea40623     	sb	a0, -0x14(s0)
   1384c: 03459513     	slli	a0, a1, 0x34
   13850: 9169         	srli	a0, a0, 0x3a
   13852: 08050513     	addi	a0, a0, 0x80
   13856: fea406a3     	sb	a0, -0x13(s0)
   1385a: 03f5f513     	andi	a0, a1, 0x3f
   1385e: 08050513     	addi	a0, a0, 0x80
   13862: fea40723     	sb	a0, -0x12(s0)
   13866: 460d         	li	a2, 0x3
   13868: a825         	j	0x138a0 <_ZN43_$LT$char$u20$as$u20$core..fmt..Display$GT$3fmt17h463f284830fb16c3E+0xb8>
   1386a: 0125d513     	srli	a0, a1, 0x12
   1386e: 0f056513     	ori	a0, a0, 0xf0
   13872: fea40623     	sb	a0, -0x14(s0)
   13876: 02e59513     	slli	a0, a1, 0x2e
   1387a: 9169         	srli	a0, a0, 0x3a
   1387c: 08050513     	addi	a0, a0, 0x80
   13880: fea406a3     	sb	a0, -0x13(s0)
   13884: 03459513     	slli	a0, a1, 0x34
   13888: 9169         	srli	a0, a0, 0x3a
   1388a: 08050513     	addi	a0, a0, 0x80
   1388e: fea40723     	sb	a0, -0x12(s0)
   13892: 03f5f513     	andi	a0, a1, 0x3f
   13896: 08050513     	addi	a0, a0, 0x80
   1389a: fea407a3     	sb	a0, -0x11(s0)
   1389e: 4611         	li	a2, 0x4
   138a0: fec40593     	addi	a1, s0, -0x14
   138a4: 8536         	mv	a0, a3
   138a6: 00000097     	auipc	ra, 0x0
   138aa: b38080e7     	jalr	-0x4c8(ra) <_ZN4core3fmt9Formatter3pad17h1bf48da13036451eE>
   138ae: 60e2         	ld	ra, 0x18(sp)
   138b0: 6442         	ld	s0, 0x10(sp)
   138b2: 6105         	addi	sp, sp, 0x20
   138b4: 8082         	ret

00000000000138b6 <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E>:
   138b6: 1141         	addi	sp, sp, -0x10
   138b8: e406         	sd	ra, 0x8(sp)
   138ba: e022         	sd	s0, 0x0(sp)
   138bc: 0800         	addi	s0, sp, 0x10
   138be: 00758693     	addi	a3, a1, 0x7
   138c2: 9ae1         	andi	a3, a3, -0x8
   138c4: 02b68a63     	beq	a3, a1, 0x138f8 <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E+0x42>
   138c8: 40b68333     	sub	t1, a3, a1
   138cc: 00c36363     	bltu	t1, a2, 0x138d2 <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E+0x1c>
   138d0: 8332         	mv	t1, a2
   138d2: 02030363     	beqz	t1, 0x138f8 <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E+0x42>
   138d6: 4701         	li	a4, 0x0
   138d8: 40600833     	neg	a6, t1
   138dc: 87ae         	mv	a5, a1
   138de: 0007c683     	lbu	a3, 0x0(a5)
   138e2: 08a68c63     	beq	a3, a0, 0x1397a <.Lpcrel_hi647+0x74>
   138e6: 177d         	addi	a4, a4, -0x1
   138e8: 0785         	addi	a5, a5, 0x1
   138ea: fee81ae3     	bne	a6, a4, 0x138de <_ZN4core5slice6memchr14memchr_aligned17h90129fc8993e8608E+0x28>
   138ee: ff060813     	addi	a6, a2, -0x10
   138f2: 00687663     	bgeu	a6, t1, 0x138fe <.Lpcrel_hi646>
   138f6: a891         	j	0x1394a <.Lpcrel_hi647+0x44>
   138f8: 4301         	li	t1, 0x0
   138fa: ff060813     	addi	a6, a2, -0x10

00000000000138fe <.Lpcrel_hi646>:
   138fe: 00002717     	auipc	a4, 0x2
   13902: a5273883     	ld	a7, -0x5ae(a4)

0000000000013906 <.Lpcrel_hi647>:
   13906: 00002717     	auipc	a4, 0x2
   1390a: a4273383     	ld	t2, -0x5be(a4)
   1390e: 00188793     	addi	a5, a7, 0x1
   13912: 02f50e33     	mul	t3, a0, a5
   13916: 00858293     	addi	t0, a1, 0x8
   1391a: 006586b3     	add	a3, a1, t1
   1391e: 6294         	ld	a3, 0x0(a3)
   13920: 00628733     	add	a4, t0, t1
   13924: 6318         	ld	a4, 0x0(a4)
   13926: 01c6c6b3     	xor	a3, a3, t3
   1392a: 40d887b3     	sub	a5, a7, a3
   1392e: 8edd         	or	a3, a3, a5
   13930: 01c74733     	xor	a4, a4, t3
   13934: 40e887b3     	sub	a5, a7, a4
   13938: 8f5d         	or	a4, a4, a5
   1393a: 8ef9         	and	a3, a3, a4
   1393c: 0076f6b3     	and	a3, a3, t2
   13940: 00769563     	bne	a3, t2, 0x1394a <.Lpcrel_hi647+0x44>
   13944: 0341         	addi	t1, t1, 0x10
   13946: fc687ae3     	bgeu	a6, t1, 0x1391a <.Lpcrel_hi647+0x14>
   1394a: 02c30063     	beq	t1, a2, 0x1396a <.Lpcrel_hi647+0x64>
   1394e: 00658733     	add	a4, a1, t1
   13952: 406005b3     	neg	a1, t1
   13956: 40c00633     	neg	a2, a2
   1395a: 00074683     	lbu	a3, 0x0(a4)
   1395e: 00a68b63     	beq	a3, a0, 0x13974 <.Lpcrel_hi647+0x6e>
   13962: 15fd         	addi	a1, a1, -0x1
   13964: 0705         	addi	a4, a4, 0x1
   13966: feb61ae3     	bne	a2, a1, 0x1395a <.Lpcrel_hi647+0x54>
   1396a: 4501         	li	a0, 0x0
   1396c: 60a2         	ld	ra, 0x8(sp)
   1396e: 6402         	ld	s0, 0x0(sp)
   13970: 0141         	addi	sp, sp, 0x10
   13972: 8082         	ret
   13974: 40b005b3     	neg	a1, a1
   13978: a019         	j	0x1397e <.Lpcrel_hi647+0x78>
   1397a: 40e005b3     	neg	a1, a4
   1397e: 4505         	li	a0, 0x1
   13980: 60a2         	ld	ra, 0x8(sp)
   13982: 6402         	ld	s0, 0x0(sp)
   13984: 0141         	addi	sp, sp, 0x10
   13986: 8082         	ret

0000000000013988 <_ZN4core5slice5index24slice_end_index_len_fail17h6d0be8bee959f757E>:
   13988: 1141         	addi	sp, sp, -0x10
   1398a: e406         	sd	ra, 0x8(sp)
   1398c: e022         	sd	s0, 0x0(sp)
   1398e: 0800         	addi	s0, sp, 0x10
   13990: 00001097     	auipc	ra, 0x1
   13994: ac8080e7     	jalr	-0x538(ra) <_ZN4core5slice5index24slice_end_index_len_fail8do_panic7runtime17h77168e5889f9c243E>

0000000000013998 <_ZN4core5slice5index22slice_index_order_fail17h37263f3371ee24c8E>:
   13998: 1141         	addi	sp, sp, -0x10
   1399a: e406         	sd	ra, 0x8(sp)
   1399c: e022         	sd	s0, 0x0(sp)
   1399e: 0800         	addi	s0, sp, 0x10
   139a0: 00001097     	auipc	ra, 0x1
   139a4: b18080e7     	jalr	-0x4e8(ra) <_ZN4core5slice5index22slice_index_order_fail8do_panic7runtime17h39c7f1be29aceb7cE>

00000000000139a8 <_ZN4core3str8converts9from_utf817ha5989fec8d0859a2E>:
   139a8: 7159         	addi	sp, sp, -0x70
   139aa: f486         	sd	ra, 0x68(sp)
   139ac: f0a2         	sd	s0, 0x60(sp)
   139ae: eca6         	sd	s1, 0x58(sp)
   139b0: e8ca         	sd	s2, 0x50(sp)
   139b2: e4ce         	sd	s3, 0x48(sp)
   139b4: e0d2         	sd	s4, 0x40(sp)
   139b6: fc56         	sd	s5, 0x38(sp)
   139b8: f85a         	sd	s6, 0x30(sp)
   139ba: f45e         	sd	s7, 0x28(sp)
   139bc: f062         	sd	s8, 0x20(sp)
   139be: ec66         	sd	s9, 0x18(sp)
   139c0: e86a         	sd	s10, 0x10(sp)
   139c2: e46e         	sd	s11, 0x8(sp)
   139c4: 1880         	addi	s0, sp, 0x70
   139c6: 1a060f63     	beqz	a2, 0x13b84 <.Lpcrel_hi667+0x136>
   139ca: 4681         	li	a3, 0x0
   139cc: ff160713     	addi	a4, a2, -0xf
   139d0: 00e637b3     	sltu	a5, a2, a4
   139d4: 17fd         	addi	a5, a5, -0x1
   139d6: 00e7fe33     	and	t3, a5, a4
   139da: 00758793     	addi	a5, a1, 0x7
   139de: 9be1         	andi	a5, a5, -0x8
   139e0: 40b78833     	sub	a6, a5, a1
   139e4: 00858c93     	addi	s9, a1, 0x8
   139e8: 40c008b3     	neg	a7, a2

00000000000139ec <.Lpcrel_hi666>:
   139ec: 00002797     	auipc	a5, 0x2
   139f0: 5c778293     	addi	t0, a5, 0x5c7
   139f4: 4311         	li	t1, 0x4
   139f6: 0f000393     	li	t2, 0xf0
   139fa: fbf00e93     	li	t4, -0x41
   139fe: 0f400f13     	li	t5, 0xf4
   13a02: f8f00f93     	li	t6, -0x71
   13a06: 4989         	li	s3, 0x2
   13a08: fc000913     	li	s2, -0x40
   13a0c: 4d0d         	li	s10, 0x3
   13a0e: 0e000a13     	li	s4, 0xe0
   13a12: 0a000a93     	li	s5, 0xa0
   13a16: 0ed00b13     	li	s6, 0xed
   13a1a: f9f00b93     	li	s7, -0x61
   13a1e: 4c31         	li	s8, 0xc
   13a20: 4d85         	li	s11, 0x1
   13a22: a021         	j	0x13a2a <.Lpcrel_hi666+0x3e>
   13a24: 0685         	addi	a3, a3, 0x1
   13a26: 14c6ff63     	bgeu	a3, a2, 0x13b84 <.Lpcrel_hi667+0x136>
   13a2a: 00d587b3     	add	a5, a1, a3
   13a2e: 00078783     	lb	a5, 0x0(a5)
   13a32: 0407c663     	bltz	a5, 0x13a7e <.Lpcrel_hi667+0x30>
   13a36: 40d807bb     	subw	a5, a6, a3
   13a3a: 8b9d         	andi	a5, a5, 0x7
   13a3c: f7e5         	bnez	a5, 0x13a24 <.Lpcrel_hi666+0x38>
   13a3e: 03c6f263     	bgeu	a3, t3, 0x13a62 <.Lpcrel_hi667+0x14>
   13a42: 00d587b3     	add	a5, a1, a3
   13a46: 639c         	ld	a5, 0x0(a5)
   13a48: 00dc84b3     	add	s1, s9, a3
   13a4c: 6084         	ld	s1, 0x0(s1)

0000000000013a4e <.Lpcrel_hi667>:
   13a4e: 00002717     	auipc	a4, 0x2
   13a52: 8fa73703     	ld	a4, -0x706(a4)
   13a56: 8fc5         	or	a5, a5, s1
   13a58: 8f7d         	and	a4, a4, a5
   13a5a: e701         	bnez	a4, 0x13a62 <.Lpcrel_hi667+0x14>
   13a5c: 06c1         	addi	a3, a3, 0x10
   13a5e: ffc6e2e3     	bltu	a3, t3, 0x13a42 <.Lpcrel_hi666+0x56>
   13a62: 08c6fa63     	bgeu	a3, a2, 0x13af6 <.Lpcrel_hi667+0xa8>
   13a66: 40d004b3     	neg	s1, a3
   13a6a: 96ae         	add	a3, a3, a1
   13a6c: 00068703     	lb	a4, 0x0(a3)
   13a70: 08074163     	bltz	a4, 0x13af2 <.Lpcrel_hi667+0xa4>
   13a74: 14fd         	addi	s1, s1, -0x1
   13a76: 0685         	addi	a3, a3, 0x1
   13a78: fe989ae3     	bne	a7, s1, 0x13a6c <.Lpcrel_hi667+0x1e>
   13a7c: a221         	j	0x13b84 <.Lpcrel_hi667+0x136>
   13a7e: 0ff7f493     	andi	s1, a5, 0xff
   13a82: 00928733     	add	a4, t0, s1
   13a86: 00074783     	lbu	a5, 0x0(a4)
   13a8a: 04678363     	beq	a5, t1, 0x13ad0 <.Lpcrel_hi667+0x82>
   13a8e: 03a78063     	beq	a5, s10, 0x13aae <.Lpcrel_hi667+0x60>
   13a92: 0f379f63     	bne	a5, s3, 0x13b90 <.Lpcrel_hi667+0x142>
   13a96: 00168493     	addi	s1, a3, 0x1
   13a9a: 0ec4f963     	bgeu	s1, a2, 0x13b8c <.Lpcrel_hi667+0x13e>
   13a9e: 00958733     	add	a4, a1, s1
   13aa2: 00070703     	lb	a4, 0x0(a4)
   13aa6: 4085         	li	ra, 0x1
   13aa8: 0ceeda63     	bge	t4, a4, 0x13b7c <.Lpcrel_hi667+0x12e>
   13aac: a0fd         	j	0x13b9a <.Lpcrel_hi667+0x14c>
   13aae: 00168793     	addi	a5, a3, 0x1
   13ab2: 0cc7fd63     	bgeu	a5, a2, 0x13b8c <.Lpcrel_hi667+0x13e>
   13ab6: 97ae         	add	a5, a5, a1
   13ab8: 0007c083     	lbu	ra, 0x0(a5)
   13abc: 05448063     	beq	s1, s4, 0x13afc <.Lpcrel_hi667+0xae>
   13ac0: 10e2         	slli	ra, ra, 0x38
   13ac2: 05649963     	bne	s1, s6, 0x13b14 <.Lpcrel_hi667+0xc6>
   13ac6: 4380d713     	srai	a4, ra, 0x38
   13aca: 06ebd363     	bge	s7, a4, 0x13b30 <.Lpcrel_hi667+0xe2>
   13ace: a0c9         	j	0x13b90 <.Lpcrel_hi667+0x142>
   13ad0: 00168793     	addi	a5, a3, 0x1
   13ad4: 0ac7fc63     	bgeu	a5, a2, 0x13b8c <.Lpcrel_hi667+0x13e>
   13ad8: 97ae         	add	a5, a5, a1
   13ada: 0007c083     	lbu	ra, 0x0(a5)
   13ade: 02748463     	beq	s1, t2, 0x13b06 <.Lpcrel_hi667+0xb8>
   13ae2: 10e2         	slli	ra, ra, 0x38
   13ae4: 07e49163     	bne	s1, t5, 0x13b46 <.Lpcrel_hi667+0xf8>
   13ae8: 4380d713     	srai	a4, ra, 0x38
   13aec: 06efd563     	bge	t6, a4, 0x13b56 <.Lpcrel_hi667+0x108>
   13af0: a045         	j	0x13b90 <.Lpcrel_hi667+0x142>
   13af2: 409006b3     	neg	a3, s1
   13af6: f2c6eae3     	bltu	a3, a2, 0x13a2a <.Lpcrel_hi666+0x3e>
   13afa: a069         	j	0x13b84 <.Lpcrel_hi667+0x136>
   13afc: 0e00f713     	andi	a4, ra, 0xe0
   13b00: 03570863     	beq	a4, s5, 0x13b30 <.Lpcrel_hi667+0xe2>
   13b04: a071         	j	0x13b90 <.Lpcrel_hi667+0x142>
   13b06: f7008713     	addi	a4, ra, -0x90
   13b0a: 03000793     	li	a5, 0x30
   13b0e: 04f76463     	bltu	a4, a5, 0x13b56 <.Lpcrel_hi667+0x108>
   13b12: a8bd         	j	0x13b90 <.Lpcrel_hi667+0x142>
   13b14: f1f48713     	addi	a4, s1, -0xe1
   13b18: 01876863     	bltu	a4, s8, 0x13b28 <.Lpcrel_hi667+0xda>
   13b1c: 0fe4f713     	andi	a4, s1, 0xfe
   13b20: 0ee00793     	li	a5, 0xee
   13b24: 06f71663     	bne	a4, a5, 0x13b90 <.Lpcrel_hi667+0x142>
   13b28: 4380d713     	srai	a4, ra, 0x38
   13b2c: 07275263     	bge	a4, s2, 0x13b90 <.Lpcrel_hi667+0x142>
   13b30: 00268493     	addi	s1, a3, 0x2
   13b34: 04c4fc63     	bgeu	s1, a2, 0x13b8c <.Lpcrel_hi667+0x13e>
   13b38: 00958733     	add	a4, a1, s1
   13b3c: 00070703     	lb	a4, 0x0(a4)
   13b40: 02eede63     	bge	t4, a4, 0x13b7c <.Lpcrel_hi667+0x12e>
   13b44: a881         	j	0x13b94 <.Lpcrel_hi667+0x146>
   13b46: f0f48713     	addi	a4, s1, -0xf1
   13b4a: 04e9e363     	bltu	s3, a4, 0x13b90 <.Lpcrel_hi667+0x142>
   13b4e: 4380d713     	srai	a4, ra, 0x38
   13b52: 03275f63     	bge	a4, s2, 0x13b90 <.Lpcrel_hi667+0x142>
   13b56: 00268793     	addi	a5, a3, 0x2
   13b5a: 02c7f963     	bgeu	a5, a2, 0x13b8c <.Lpcrel_hi667+0x13e>
   13b5e: 97ae         	add	a5, a5, a1
   13b60: 00078703     	lb	a4, 0x0(a5)
   13b64: 02eec863     	blt	t4, a4, 0x13b94 <.Lpcrel_hi667+0x146>
   13b68: 00368493     	addi	s1, a3, 0x3
   13b6c: 02c4f063     	bgeu	s1, a2, 0x13b8c <.Lpcrel_hi667+0x13e>
   13b70: 00958733     	add	a4, a1, s1
   13b74: 00070703     	lb	a4, 0x0(a4)
   13b78: 02eec063     	blt	t4, a4, 0x13b98 <.Lpcrel_hi667+0x14a>
   13b7c: 00148693     	addi	a3, s1, 0x1
   13b80: eac6e5e3     	bltu	a3, a2, 0x13a2a <.Lpcrel_hi666+0x3e>
   13b84: 4681         	li	a3, 0x0
   13b86: e50c         	sd	a1, 0x8(a0)
   13b88: e910         	sd	a2, 0x10(a0)
   13b8a: a831         	j	0x13ba6 <.Lpcrel_hi667+0x158>
   13b8c: 4d81         	li	s11, 0x0
   13b8e: a031         	j	0x13b9a <.Lpcrel_hi667+0x14c>
   13b90: 4085         	li	ra, 0x1
   13b92: a021         	j	0x13b9a <.Lpcrel_hi667+0x14c>
   13b94: 4089         	li	ra, 0x2
   13b96: a011         	j	0x13b9a <.Lpcrel_hi667+0x14c>
   13b98: 408d         	li	ra, 0x3
   13b9a: e514         	sd	a3, 0x8(a0)
   13b9c: 01b50823     	sb	s11, 0x10(a0)
   13ba0: 001508a3     	sb	ra, 0x11(a0)
   13ba4: 4685         	li	a3, 0x1
   13ba6: e114         	sd	a3, 0x0(a0)
   13ba8: 70a6         	ld	ra, 0x68(sp)
   13baa: 7406         	ld	s0, 0x60(sp)
   13bac: 64e6         	ld	s1, 0x58(sp)
   13bae: 6946         	ld	s2, 0x50(sp)
   13bb0: 69a6         	ld	s3, 0x48(sp)
   13bb2: 6a06         	ld	s4, 0x40(sp)
   13bb4: 7ae2         	ld	s5, 0x38(sp)
   13bb6: 7b42         	ld	s6, 0x30(sp)
   13bb8: 7ba2         	ld	s7, 0x28(sp)
   13bba: 7c02         	ld	s8, 0x20(sp)
   13bbc: 6ce2         	ld	s9, 0x18(sp)
   13bbe: 6d42         	ld	s10, 0x10(sp)
   13bc0: 6da2         	ld	s11, 0x8(sp)
   13bc2: 6165         	addi	sp, sp, 0x70
   13bc4: 8082         	ret

0000000000013bc6 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE>:
   13bc6: 1141         	addi	sp, sp, -0x10
   13bc8: e406         	sd	ra, 0x8(sp)
   13bca: e022         	sd	s0, 0x0(sp)
   13bcc: 0800         	addi	s0, sp, 0x10
   13bce: 862a         	mv	a2, a0
   13bd0: 00750793     	addi	a5, a0, 0x7
   13bd4: 9be1         	andi	a5, a5, -0x8
   13bd6: 40a786b3     	sub	a3, a5, a0
   13bda: 02d5f363     	bgeu	a1, a3, 0x13c00 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x3a>
   13bde: 4501         	li	a0, 0x0
   13be0: cd81         	beqz	a1, 0x13bf8 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x32>
   13be2: 95b2         	add	a1, a1, a2
   13be4: 00060683     	lb	a3, 0x0(a2)
   13be8: fc06a693     	slti	a3, a3, -0x40
   13bec: 0016c693     	xori	a3, a3, 0x1
   13bf0: 0605         	addi	a2, a2, 0x1
   13bf2: 9536         	add	a0, a0, a3
   13bf4: feb618e3     	bne	a2, a1, 0x13be4 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x1e>
   13bf8: 60a2         	ld	ra, 0x8(sp)
   13bfa: 6402         	ld	s0, 0x0(sp)
   13bfc: 0141         	addi	sp, sp, 0x10
   13bfe: 8082         	ret
   13c00: 40d58733     	sub	a4, a1, a3
   13c04: 00375313     	srli	t1, a4, 0x3
   13c08: fc030be3     	beqz	t1, 0x13bde <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x18>
   13c0c: 96b2         	add	a3, a3, a2
   13c0e: 00777813     	andi	a6, a4, 0x7
   13c12: 4501         	li	a0, 0x0
   13c14: 00c78c63     	beq	a5, a2, 0x13c2c <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x66>
   13c18: 00060583     	lb	a1, 0x0(a2)
   13c1c: fc05a593     	slti	a1, a1, -0x40
   13c20: 0015c593     	xori	a1, a1, 0x1
   13c24: 0605         	addi	a2, a2, 0x1
   13c26: 952e         	add	a0, a0, a1
   13c28: fed618e3     	bne	a2, a3, 0x13c18 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x52>
   13c2c: 4601         	li	a2, 0x0
   13c2e: 02080263     	beqz	a6, 0x13c52 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x8c>
   13c32: ff877593     	andi	a1, a4, -0x8
   13c36: 00b68733     	add	a4, a3, a1
   13c3a: 95be         	add	a1, a1, a5
   13c3c: 95c2         	add	a1, a1, a6
   13c3e: 00070783     	lb	a5, 0x0(a4)
   13c42: fc07a793     	slti	a5, a5, -0x40
   13c46: 0017c793     	xori	a5, a5, 0x1
   13c4a: 0705         	addi	a4, a4, 0x1
   13c4c: 963e         	add	a2, a2, a5
   13c4e: feb718e3     	bne	a4, a1, 0x13c3e <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x78>
   13c52: 010105b7     	lui	a1, 0x1010
   13c56: 1015859b     	addiw	a1, a1, 0x101
   13c5a: 02059713     	slli	a4, a1, 0x20
   13c5e: 00e58fb3     	add	t6, a1, a4
   13c62: 00ff0737     	lui	a4, 0xff0
   13c66: 0ff7089b     	addiw	a7, a4, 0xff
   13c6a: 02089713     	slli	a4, a7, 0x20
   13c6e: 98ba         	add	a7, a7, a4
   13c70: 6741         	lui	a4, 0x10
   13c72: 2705         	addiw	a4, a4, 0x1
   13c74: 02071813     	slli	a6, a4, 0x20
   13c78: 983a         	add	a6, a6, a4
   13c7a: 9532         	add	a0, a0, a2
   13c7c: 4291         	li	t0, 0x4
   13c7e: a015         	j	0x13ca2 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0xdc>
   13c80: 006e06b3     	add	a3, t3, t1
   13c84: 407e8333     	sub	t1, t4, t2
   13c88: 0033f593     	andi	a1, t2, 0x3
   13c8c: 0117f633     	and	a2, a5, a7
   13c90: 83a1         	srli	a5, a5, 0x8
   13c92: 0117f733     	and	a4, a5, a7
   13c96: 963a         	add	a2, a2, a4
   13c98: 03060633     	mul	a2, a2, a6
   13c9c: 9241         	srli	a2, a2, 0x30
   13c9e: 9532         	add	a0, a0, a2
   13ca0: edbd         	bnez	a1, 0x13d1e <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x158>
   13ca2: f4030be3     	beqz	t1, 0x13bf8 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x32>
   13ca6: 8e9a         	mv	t4, t1
   13ca8: 8e36         	mv	t3, a3
   13caa: 0c000613     	li	a2, 0xc0
   13cae: 839a         	mv	t2, t1
   13cb0: 00c36463     	bltu	t1, a2, 0x13cb8 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0xf2>
   13cb4: 0c000393     	li	t2, 0xc0
   13cb8: 00339313     	slli	t1, t2, 0x3
   13cbc: 4781         	li	a5, 0x0
   13cbe: fc5ee1e3     	bltu	t4, t0, 0x13c80 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0xba>
   13cc2: 7e037613     	andi	a2, t1, 0x7e0
   13cc6: 00ce0f33     	add	t5, t3, a2
   13cca: 86f2         	mv	a3, t3
   13ccc: 6298         	ld	a4, 0x0(a3)
   13cce: fff74613     	not	a2, a4
   13cd2: 821d         	srli	a2, a2, 0x7
   13cd4: 8319         	srli	a4, a4, 0x6
   13cd6: 668c         	ld	a1, 0x8(a3)
   13cd8: 8e59         	or	a2, a2, a4
   13cda: 01f67633     	and	a2, a2, t6
   13cde: 963e         	add	a2, a2, a5
   13ce0: fff5c713     	not	a4, a1
   13ce4: 831d         	srli	a4, a4, 0x7
   13ce6: 6a9c         	ld	a5, 0x10(a3)
   13ce8: 8199         	srli	a1, a1, 0x6
   13cea: 8dd9         	or	a1, a1, a4
   13cec: 01f5f5b3     	and	a1, a1, t6
   13cf0: fff7c713     	not	a4, a5
   13cf4: 831d         	srli	a4, a4, 0x7
   13cf6: 8399         	srli	a5, a5, 0x6
   13cf8: 8f5d         	or	a4, a4, a5
   13cfa: 6e9c         	ld	a5, 0x18(a3)
   13cfc: 01f77733     	and	a4, a4, t6
   13d00: 95ba         	add	a1, a1, a4
   13d02: 95b2         	add	a1, a1, a2
   13d04: fff7c613     	not	a2, a5
   13d08: 821d         	srli	a2, a2, 0x7
   13d0a: 8399         	srli	a5, a5, 0x6
   13d0c: 8e5d         	or	a2, a2, a5
   13d0e: 01f677b3     	and	a5, a2, t6
   13d12: 02068693     	addi	a3, a3, 0x20
   13d16: 97ae         	add	a5, a5, a1
   13d18: fbe69ae3     	bne	a3, t5, 0x13ccc <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x106>
   13d1c: b795         	j	0x13c80 <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0xba>
   13d1e: 4601         	li	a2, 0x0
   13d20: 0fc3f593     	andi	a1, t2, 0xfc
   13d24: 058e         	slli	a1, a1, 0x3
   13d26: 9e2e         	add	t3, t3, a1
   13d28: 0c0eb593     	sltiu	a1, t4, 0xc0
   13d2c: 40b005bb     	negw	a1, a1
   13d30: 00bef5b3     	and	a1, t4, a1
   13d34: 898d         	andi	a1, a1, 0x3
   13d36: 00359693     	slli	a3, a1, 0x3
   13d3a: 000e3583     	ld	a1, 0x0(t3)
   13d3e: 0e21         	addi	t3, t3, 0x8
   13d40: fff5c713     	not	a4, a1
   13d44: 831d         	srli	a4, a4, 0x7
   13d46: 8199         	srli	a1, a1, 0x6
   13d48: 8dd9         	or	a1, a1, a4
   13d4a: 01f5f5b3     	and	a1, a1, t6
   13d4e: 16e1         	addi	a3, a3, -0x8
   13d50: 962e         	add	a2, a2, a1
   13d52: f6e5         	bnez	a3, 0x13d3a <_ZN4core3str5count14do_count_chars17h17791e0a5f685f0cE+0x174>
   13d54: 011675b3     	and	a1, a2, a7
   13d58: 8221         	srli	a2, a2, 0x8
   13d5a: 01167633     	and	a2, a2, a7
   13d5e: 95b2         	add	a1, a1, a2
   13d60: 030585b3     	mul	a1, a1, a6
   13d64: 91c1         	srli	a1, a1, 0x30
   13d66: 952e         	add	a0, a0, a1
   13d68: 60a2         	ld	ra, 0x8(sp)
   13d6a: 6402         	ld	s0, 0x0(sp)
   13d6c: 0141         	addi	sp, sp, 0x10
   13d6e: 8082         	ret

0000000000013d70 <_ZN73_$LT$core..num..nonzero..NonZero$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4a76621c4b3176a1E>:
   13d70: 7175         	addi	sp, sp, -0x90
   13d72: e506         	sd	ra, 0x88(sp)
   13d74: e122         	sd	s0, 0x80(sp)
   13d76: 0900         	addi	s0, sp, 0x90
   13d78: 882e         	mv	a6, a1
   13d7a: 0245e583     	lwu	a1, 0x24(a1)
   13d7e: 6108         	ld	a0, 0x0(a0)
   13d80: 0105f613     	andi	a2, a1, 0x10
   13d84: ee09         	bnez	a2, 0x13d9e <_ZN73_$LT$core..num..nonzero..NonZero$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4a76621c4b3176a1E+0x2e>
   13d86: 0205f593     	andi	a1, a1, 0x20
   13d8a: e9a1         	bnez	a1, 0x13dda <.Lpcrel_hi897+0xa>
   13d8c: 4585         	li	a1, 0x1
   13d8e: 8642         	mv	a2, a6
   13d90: 60aa         	ld	ra, 0x88(sp)
   13d92: 640a         	ld	s0, 0x80(sp)
   13d94: 6149         	addi	sp, sp, 0x90
   13d96: 00000317     	auipc	t1, 0x0
   13d9a: 53030067     	jr	0x530(t1) <_ZN4core3fmt3num3imp21_$LT$impl$u20$u64$GT$4_fmt17he678ad54334687f6E>
   13d9e: 4781         	li	a5, 0x0
   13da0: fef40593     	addi	a1, s0, -0x11
   13da4: 4629         	li	a2, 0xa
   13da6: a809         	j	0x13db8 <_ZN73_$LT$core..num..nonzero..NonZero$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4a76621c4b3176a1E+0x48>
   13da8: 05768693     	addi	a3, a3, 0x57
   13dac: 8111         	srli	a0, a0, 0x4
   13dae: 00d58023     	sb	a3, 0x0(a1)
   13db2: 0785         	addi	a5, a5, 0x1
   13db4: 15fd         	addi	a1, a1, -0x1
   13db6: c901         	beqz	a0, 0x13dc6 <_ZN73_$LT$core..num..nonzero..NonZero$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4a76621c4b3176a1E+0x56>
   13db8: 00f57693     	andi	a3, a0, 0xf
   13dbc: fec6f6e3     	bgeu	a3, a2, 0x13da8 <_ZN73_$LT$core..num..nonzero..NonZero$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4a76621c4b3176a1E+0x38>
   13dc0: 03068693     	addi	a3, a3, 0x30
   13dc4: b7e5         	j	0x13dac <_ZN73_$LT$core..num..nonzero..NonZero$LT$T$GT$$u20$as$u20$core..fmt..Debug$GT$3fmt17h4a76621c4b3176a1E+0x3c>
   13dc6: f7040513     	addi	a0, s0, -0x90
   13dca: 8d1d         	sub	a0, a0, a5
   13dcc: 08050713     	addi	a4, a0, 0x80

0000000000013dd0 <.Lpcrel_hi897>:
   13dd0: 00002517     	auipc	a0, 0x2
   13dd4: 11950613     	addi	a2, a0, 0x119
   13dd8: a835         	j	0x13e14 <.Lpcrel_hi898+0x8>
   13dda: 4781         	li	a5, 0x0
   13ddc: fef40593     	addi	a1, s0, -0x11
   13de0: 4629         	li	a2, 0xa
   13de2: a809         	j	0x13df4 <.Lpcrel_hi897+0x24>
   13de4: 03768693     	addi	a3, a3, 0x37
   13de8: 8111         	srli	a0, a0, 0x4
   13dea: 00d58023     	sb	a3, 0x0(a1)
   13dee: 0785         	addi	a5, a5, 0x1
   13df0: 15fd         	addi	a1, a1, -0x1
   13df2: c901         	beqz	a0, 0x13e02 <.Lpcrel_hi897+0x32>
   13df4: 00f57693     	andi	a3, a0, 0xf
   13df8: fec6f6e3     	bgeu	a3, a2, 0x13de4 <.Lpcrel_hi897+0x14>
   13dfc: 03068693     	addi	a3, a3, 0x30
   13e00: b7e5         	j	0x13de8 <.Lpcrel_hi897+0x18>
   13e02: f7040513     	addi	a0, s0, -0x90
   13e06: 8d1d         	sub	a0, a0, a5
   13e08: 08050713     	addi	a4, a0, 0x80

0000000000013e0c <.Lpcrel_hi898>:
   13e0c: 00002517     	auipc	a0, 0x2
   13e10: 0dd50613     	addi	a2, a0, 0xdd
   13e14: 4585         	li	a1, 0x1
   13e16: 4689         	li	a3, 0x2
   13e18: 8542         	mv	a0, a6
   13e1a: fffff097     	auipc	ra, 0xfffff
   13e1e: 340080e7     	jalr	0x340(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   13e22: 60aa         	ld	ra, 0x88(sp)
   13e24: 640a         	ld	s0, 0x80(sp)
   13e26: 6149         	addi	sp, sp, 0x90
   13e28: 8082         	ret

0000000000013e2a <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h75051e47a1d83752E>:
   13e2a: 7175         	addi	sp, sp, -0x90
   13e2c: e506         	sd	ra, 0x88(sp)
   13e2e: e122         	sd	s0, 0x80(sp)
   13e30: 0900         	addi	s0, sp, 0x90
   13e32: 00054603     	lbu	a2, 0x0(a0)
   13e36: 852e         	mv	a0, a1
   13e38: 4781         	li	a5, 0x0
   13e3a: fef40593     	addi	a1, s0, -0x11
   13e3e: 46a9         	li	a3, 0xa
   13e40: a809         	j	0x13e52 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h75051e47a1d83752E+0x28>
   13e42: 05770713     	addi	a4, a4, 0x57
   13e46: 8211         	srli	a2, a2, 0x4
   13e48: 00e58023     	sb	a4, 0x0(a1)
   13e4c: 0785         	addi	a5, a5, 0x1
   13e4e: 15fd         	addi	a1, a1, -0x1
   13e50: ca01         	beqz	a2, 0x13e60 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h75051e47a1d83752E+0x36>
   13e52: 00f67713     	andi	a4, a2, 0xf
   13e56: fed776e3     	bgeu	a4, a3, 0x13e42 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h75051e47a1d83752E+0x18>
   13e5a: 03070713     	addi	a4, a4, 0x30
   13e5e: b7e5         	j	0x13e46 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$u8$GT$3fmt17h75051e47a1d83752E+0x1c>
   13e60: f7040593     	addi	a1, s0, -0x90
   13e64: 8d9d         	sub	a1, a1, a5
   13e66: 08058713     	addi	a4, a1, 0x80

0000000000013e6a <.Lpcrel_hi991>:
   13e6a: 00002597     	auipc	a1, 0x2
   13e6e: 07f58613     	addi	a2, a1, 0x7f
   13e72: 4585         	li	a1, 0x1
   13e74: 4689         	li	a3, 0x2
   13e76: fffff097     	auipc	ra, 0xfffff
   13e7a: 2e4080e7     	jalr	0x2e4(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   13e7e: 60aa         	ld	ra, 0x88(sp)
   13e80: 640a         	ld	s0, 0x80(sp)
   13e82: 6149         	addi	sp, sp, 0x90
   13e84: 8082         	ret

0000000000013e86 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h42a55981b2e0dc80E>:
   13e86: 7175         	addi	sp, sp, -0x90
   13e88: e506         	sd	ra, 0x88(sp)
   13e8a: e122         	sd	s0, 0x80(sp)
   13e8c: 0900         	addi	s0, sp, 0x90
   13e8e: 00054603     	lbu	a2, 0x0(a0)
   13e92: 852e         	mv	a0, a1
   13e94: 4781         	li	a5, 0x0
   13e96: fef40593     	addi	a1, s0, -0x11
   13e9a: 46a9         	li	a3, 0xa
   13e9c: a809         	j	0x13eae <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h42a55981b2e0dc80E+0x28>
   13e9e: 03770713     	addi	a4, a4, 0x37
   13ea2: 8211         	srli	a2, a2, 0x4
   13ea4: 00e58023     	sb	a4, 0x0(a1)
   13ea8: 0785         	addi	a5, a5, 0x1
   13eaa: 15fd         	addi	a1, a1, -0x1
   13eac: ca01         	beqz	a2, 0x13ebc <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h42a55981b2e0dc80E+0x36>
   13eae: 00f67713     	andi	a4, a2, 0xf
   13eb2: fed776e3     	bgeu	a4, a3, 0x13e9e <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h42a55981b2e0dc80E+0x18>
   13eb6: 03070713     	addi	a4, a4, 0x30
   13eba: b7e5         	j	0x13ea2 <_ZN4core3fmt3num52_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$u8$GT$3fmt17h42a55981b2e0dc80E+0x1c>
   13ebc: f7040593     	addi	a1, s0, -0x90
   13ec0: 8d9d         	sub	a1, a1, a5
   13ec2: 08058713     	addi	a4, a1, 0x80

0000000000013ec6 <.Lpcrel_hi992>:
   13ec6: 00002597     	auipc	a1, 0x2
   13eca: 02358613     	addi	a2, a1, 0x23
   13ece: 4585         	li	a1, 0x1
   13ed0: 4689         	li	a3, 0x2
   13ed2: fffff097     	auipc	ra, 0xfffff
   13ed6: 288080e7     	jalr	0x288(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   13eda: 60aa         	ld	ra, 0x88(sp)
   13edc: 640a         	ld	s0, 0x80(sp)
   13ede: 6149         	addi	sp, sp, 0x90
   13ee0: 8082         	ret

0000000000013ee2 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E>:
   13ee2: 7175         	addi	sp, sp, -0x90
   13ee4: e506         	sd	ra, 0x88(sp)
   13ee6: e122         	sd	s0, 0x80(sp)
   13ee8: 0900         	addi	s0, sp, 0x90
   13eea: 6110         	ld	a2, 0x0(a0)
   13eec: 852e         	mv	a0, a1
   13eee: 4781         	li	a5, 0x0
   13ef0: fef40593     	addi	a1, s0, -0x11
   13ef4: 46a9         	li	a3, 0xa
   13ef6: a809         	j	0x13f08 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E+0x26>
   13ef8: 05770713     	addi	a4, a4, 0x57
   13efc: 8211         	srli	a2, a2, 0x4
   13efe: 00e58023     	sb	a4, 0x0(a1)
   13f02: 0785         	addi	a5, a5, 0x1
   13f04: 15fd         	addi	a1, a1, -0x1
   13f06: ca01         	beqz	a2, 0x13f16 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E+0x34>
   13f08: 00f67713     	andi	a4, a2, 0xf
   13f0c: fed776e3     	bgeu	a4, a3, 0x13ef8 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E+0x16>
   13f10: 03070713     	addi	a4, a4, 0x30
   13f14: b7e5         	j	0x13efc <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..LowerHex$u20$for$u20$usize$GT$3fmt17h7c6d61a51272e129E+0x1a>
   13f16: f7040593     	addi	a1, s0, -0x90
   13f1a: 8d9d         	sub	a1, a1, a5
   13f1c: 08058713     	addi	a4, a1, 0x80

0000000000013f20 <.Lpcrel_hi1003>:
   13f20: 00002597     	auipc	a1, 0x2
   13f24: fc958613     	addi	a2, a1, -0x37
   13f28: 4585         	li	a1, 0x1
   13f2a: 4689         	li	a3, 0x2
   13f2c: fffff097     	auipc	ra, 0xfffff
   13f30: 22e080e7     	jalr	0x22e(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   13f34: 60aa         	ld	ra, 0x88(sp)
   13f36: 640a         	ld	s0, 0x80(sp)
   13f38: 6149         	addi	sp, sp, 0x90
   13f3a: 8082         	ret

0000000000013f3c <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE>:
   13f3c: 7175         	addi	sp, sp, -0x90
   13f3e: e506         	sd	ra, 0x88(sp)
   13f40: e122         	sd	s0, 0x80(sp)
   13f42: 0900         	addi	s0, sp, 0x90
   13f44: 6110         	ld	a2, 0x0(a0)
   13f46: 852e         	mv	a0, a1
   13f48: 4781         	li	a5, 0x0
   13f4a: fef40593     	addi	a1, s0, -0x11
   13f4e: 46a9         	li	a3, 0xa
   13f50: a809         	j	0x13f62 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE+0x26>
   13f52: 03770713     	addi	a4, a4, 0x37
   13f56: 8211         	srli	a2, a2, 0x4
   13f58: 00e58023     	sb	a4, 0x0(a1)
   13f5c: 0785         	addi	a5, a5, 0x1
   13f5e: 15fd         	addi	a1, a1, -0x1
   13f60: ca01         	beqz	a2, 0x13f70 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE+0x34>
   13f62: 00f67713     	andi	a4, a2, 0xf
   13f66: fed776e3     	bgeu	a4, a3, 0x13f52 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE+0x16>
   13f6a: 03070713     	addi	a4, a4, 0x30
   13f6e: b7e5         	j	0x13f56 <_ZN4core3fmt3num55_$LT$impl$u20$core..fmt..UpperHex$u20$for$u20$usize$GT$3fmt17hd24a9d65678819dcE+0x1a>
   13f70: f7040593     	addi	a1, s0, -0x90
   13f74: 8d9d         	sub	a1, a1, a5
   13f76: 08058713     	addi	a4, a1, 0x80

0000000000013f7a <.Lpcrel_hi1004>:
   13f7a: 00002597     	auipc	a1, 0x2
   13f7e: f6f58613     	addi	a2, a1, -0x91
   13f82: 4585         	li	a1, 0x1
   13f84: 4689         	li	a3, 0x2
   13f86: fffff097     	auipc	ra, 0xfffff
   13f8a: 1d4080e7     	jalr	0x1d4(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   13f8e: 60aa         	ld	ra, 0x88(sp)
   13f90: 640a         	ld	s0, 0x80(sp)
   13f92: 6149         	addi	sp, sp, 0x90
   13f94: 8082         	ret

0000000000013f96 <_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u32$GT$3fmt17hb9dd612b425325e4E>:
   13f96: 7175         	addi	sp, sp, -0x90
   13f98: e506         	sd	ra, 0x88(sp)
   13f9a: e122         	sd	s0, 0x80(sp)
   13f9c: 0900         	addi	s0, sp, 0x90
   13f9e: 882e         	mv	a6, a1
   13fa0: 0245e583     	lwu	a1, 0x24(a1)
   13fa4: 0105f613     	andi	a2, a1, 0x10
   13fa8: ee11         	bnez	a2, 0x13fc4 <_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u32$GT$3fmt17hb9dd612b425325e4E+0x2e>
   13faa: 0205f593     	andi	a1, a1, 0x20
   13fae: e9b9         	bnez	a1, 0x14004 <.Lpcrel_hi1029+0xa>
   13fb0: 4108         	lw	a0, 0x0(a0)
   13fb2: 4585         	li	a1, 0x1
   13fb4: 8642         	mv	a2, a6
   13fb6: 60aa         	ld	ra, 0x88(sp)
   13fb8: 640a         	ld	s0, 0x80(sp)
   13fba: 6149         	addi	sp, sp, 0x90
   13fbc: 00000317     	auipc	t1, 0x0
   13fc0: 15c30067     	jr	0x15c(t1) <_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb343a69f7df43ef4E>
   13fc4: 4108         	lw	a0, 0x0(a0)
   13fc6: 4781         	li	a5, 0x0
   13fc8: fef40593     	addi	a1, s0, -0x11
   13fcc: 4629         	li	a2, 0xa
   13fce: a811         	j	0x13fe2 <_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u32$GT$3fmt17hb9dd612b425325e4E+0x4c>
   13fd0: 05768693     	addi	a3, a3, 0x57
   13fd4: 0045551b     	srliw	a0, a0, 0x4
   13fd8: 00d58023     	sb	a3, 0x0(a1)
   13fdc: 0785         	addi	a5, a5, 0x1
   13fde: 15fd         	addi	a1, a1, -0x1
   13fe0: c901         	beqz	a0, 0x13ff0 <_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u32$GT$3fmt17hb9dd612b425325e4E+0x5a>
   13fe2: 00f57693     	andi	a3, a0, 0xf
   13fe6: fec6f5e3     	bgeu	a3, a2, 0x13fd0 <_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u32$GT$3fmt17hb9dd612b425325e4E+0x3a>
   13fea: 03068693     	addi	a3, a3, 0x30
   13fee: b7dd         	j	0x13fd4 <_ZN4core3fmt3num50_$LT$impl$u20$core..fmt..Debug$u20$for$u20$u32$GT$3fmt17hb9dd612b425325e4E+0x3e>
   13ff0: f7040513     	addi	a0, s0, -0x90
   13ff4: 8d1d         	sub	a0, a0, a5
   13ff6: 08050713     	addi	a4, a0, 0x80

0000000000013ffa <.Lpcrel_hi1029>:
   13ffa: 00002517     	auipc	a0, 0x2
   13ffe: eef50613     	addi	a2, a0, -0x111
   14002: a081         	j	0x14042 <.Lpcrel_hi1030+0x8>
   14004: 4108         	lw	a0, 0x0(a0)
   14006: 4781         	li	a5, 0x0
   14008: fef40593     	addi	a1, s0, -0x11
   1400c: 4629         	li	a2, 0xa
   1400e: a811         	j	0x14022 <.Lpcrel_hi1029+0x28>
   14010: 03768693     	addi	a3, a3, 0x37
   14014: 0045551b     	srliw	a0, a0, 0x4
   14018: 00d58023     	sb	a3, 0x0(a1)
   1401c: 0785         	addi	a5, a5, 0x1
   1401e: 15fd         	addi	a1, a1, -0x1
   14020: c901         	beqz	a0, 0x14030 <.Lpcrel_hi1029+0x36>
   14022: 00f57693     	andi	a3, a0, 0xf
   14026: fec6f5e3     	bgeu	a3, a2, 0x14010 <.Lpcrel_hi1029+0x16>
   1402a: 03068693     	addi	a3, a3, 0x30
   1402e: b7dd         	j	0x14014 <.Lpcrel_hi1029+0x1a>
   14030: f7040513     	addi	a0, s0, -0x90
   14034: 8d1d         	sub	a0, a0, a5
   14036: 08050713     	addi	a4, a0, 0x80

000000000001403a <.Lpcrel_hi1030>:
   1403a: 00002517     	auipc	a0, 0x2
   1403e: eaf50613     	addi	a2, a0, -0x151
   14042: 4585         	li	a1, 0x1
   14044: 4689         	li	a3, 0x2
   14046: 8542         	mv	a0, a6
   14048: fffff097     	auipc	ra, 0xfffff
   1404c: 112080e7     	jalr	0x112(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   14050: 60aa         	ld	ra, 0x88(sp)
   14052: 640a         	ld	s0, 0x80(sp)
   14054: 6149         	addi	sp, sp, 0x90
   14056: 8082         	ret

0000000000014058 <_ZN4core3fmt3num3imp51_$LT$impl$u20$core..fmt..Display$u20$for$u20$u8$GT$3fmt17h47d2f4a6dbcaf6e5E>:
   14058: 1101         	addi	sp, sp, -0x20
   1405a: ec06         	sd	ra, 0x18(sp)
   1405c: e822         	sd	s0, 0x10(sp)
   1405e: 1000         	addi	s0, sp, 0x20
   14060: 00054603     	lbu	a2, 0x0(a0)
   14064: 06400693     	li	a3, 0x64
   14068: 852e         	mv	a0, a1
   1406a: 02d66d63     	bltu	a2, a3, 0x140a4 <.Lpcrel_hi1033+0x20>
   1406e: 02900593     	li	a1, 0x29
   14072: 02b605b3     	mul	a1, a2, a1
   14076: 00c5d713     	srli	a4, a1, 0xc
   1407a: 02d705b3     	mul	a1, a4, a3
   1407e: 9e0d         	subw	a2, a2, a1
   14080: 1666         	slli	a2, a2, 0x39
   14082: 9261         	srli	a2, a2, 0x38

0000000000014084 <.Lpcrel_hi1033>:
   14084: 00002597     	auipc	a1, 0x2
   14088: e6758593     	addi	a1, a1, -0x199
   1408c: 95b2         	add	a1, a1, a2
   1408e: 0015c603     	lbu	a2, 0x1(a1)
   14092: 0005c683     	lbu	a3, 0x0(a1)
   14096: 4581         	li	a1, 0x0
   14098: fec407a3     	sb	a2, -0x11(s0)
   1409c: fed40723     	sb	a3, -0x12(s0)
   140a0: 863a         	mv	a2, a4
   140a2: a029         	j	0x140ac <.Lpcrel_hi1033+0x28>
   140a4: 46a9         	li	a3, 0xa
   140a6: 4589         	li	a1, 0x2
   140a8: 00d67a63     	bgeu	a2, a3, 0x140bc <.Lpcrel_hi1033+0x38>
   140ac: 03066613     	ori	a2, a2, 0x30
   140b0: fed40693     	addi	a3, s0, -0x13
   140b4: 96ae         	add	a3, a3, a1
   140b6: 00c68023     	sb	a2, 0x0(a3)
   140ba: a00d         	j	0x140dc <.Lpcrel_hi1034+0x1c>
   140bc: 1666         	slli	a2, a2, 0x39
   140be: 9261         	srli	a2, a2, 0x38

00000000000140c0 <.Lpcrel_hi1034>:
   140c0: 00002597     	auipc	a1, 0x2
   140c4: e2b58593     	addi	a1, a1, -0x1d5
   140c8: 95b2         	add	a1, a1, a2
   140ca: 0015c603     	lbu	a2, 0x1(a1)
   140ce: 0005c583     	lbu	a1, 0x0(a1)
   140d2: fec407a3     	sb	a2, -0x11(s0)
   140d6: feb40723     	sb	a1, -0x12(s0)
   140da: 4585         	li	a1, 0x1
   140dc: fed40713     	addi	a4, s0, -0x13
   140e0: 972e         	add	a4, a4, a1
   140e2: 0035c793     	xori	a5, a1, 0x3
   140e6: 4585         	li	a1, 0x1
   140e8: 4605         	li	a2, 0x1
   140ea: 4681         	li	a3, 0x0
   140ec: fffff097     	auipc	ra, 0xfffff
   140f0: 06e080e7     	jalr	0x6e(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   140f4: 60e2         	ld	ra, 0x18(sp)
   140f6: 6442         	ld	s0, 0x10(sp)
   140f8: 6105         	addi	sp, sp, 0x20
   140fa: 8082         	ret

00000000000140fc <_ZN4core3fmt3num3imp52_$LT$impl$u20$core..fmt..Display$u20$for$u20$u32$GT$3fmt17h33120e0d85b43d27E>:
   140fc: 1141         	addi	sp, sp, -0x10
   140fe: e406         	sd	ra, 0x8(sp)
   14100: e022         	sd	s0, 0x0(sp)
   14102: 0800         	addi	s0, sp, 0x10
   14104: 4108         	lw	a0, 0x0(a0)
   14106: 862e         	mv	a2, a1
   14108: 4585         	li	a1, 0x1
   1410a: 60a2         	ld	ra, 0x8(sp)
   1410c: 6402         	ld	s0, 0x0(sp)
   1410e: 0141         	addi	sp, sp, 0x10
   14110: 00000317     	auipc	t1, 0x0
   14114: 00830067     	jr	0x8(t1) <_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb343a69f7df43ef4E>

0000000000014118 <_ZN4core3fmt3num3imp21_$LT$impl$u20$u32$GT$4_fmt17hb343a69f7df43ef4E>:
   14118: 7179         	addi	sp, sp, -0x30
   1411a: f406         	sd	ra, 0x28(sp)
   1411c: f022         	sd	s0, 0x20(sp)
   1411e: ec26         	sd	s1, 0x18(sp)
   14120: e84a         	sd	s2, 0x10(sp)
   14122: 1800         	addi	s0, sp, 0x30
   14124: 8832         	mv	a6, a2
   14126: 0045569b     	srliw	a3, a0, 0x4
   1412a: 4729         	li	a4, 0xa
   1412c: 27100793     	li	a5, 0x271

0000000000014130 <.Lpcrel_hi1041>:
   14130: 00002617     	auipc	a2, 0x2
   14134: dbb60e93     	addi	t4, a2, -0x245
   14138: 02f6f363     	bgeu	a3, a5, 0x1415e <.Lpcrel_hi1041+0x2e>
   1413c: 06300693     	li	a3, 0x63
   14140: 0aa6eb63     	bltu	a3, a0, 0x141f6 <.Lpcrel_hi1041+0xc6>
   14144: 4629         	li	a2, 0xa
   14146: 0ec57963     	bgeu	a0, a2, 0x14238 <.Lpcrel_hi1041+0x108>
   1414a: fff70693     	addi	a3, a4, -0x1
   1414e: fd640613     	addi	a2, s0, -0x2a
   14152: 9636         	add	a2, a2, a3
   14154: 03056513     	ori	a0, a0, 0x30
   14158: 00a60023     	sb	a0, 0x0(a2)
   1415c: a8ed         	j	0x14256 <.Lpcrel_hi1041+0x126>
   1415e: 4701         	li	a4, 0x0
   14160: fdc40893     	addi	a7, s0, -0x24
   14164: fde40293     	addi	t0, s0, -0x22
   14168: d1b716b7     	lui	a3, 0xd1b71
   1416c: 75968313     	addi	t1, a3, 0x759
   14170: 1302         	slli	t1, t1, 0x20
   14172: 6689         	lui	a3, 0x2
   14174: 71068e13     	addi	t3, a3, 0x710
   14178: 6685         	lui	a3, 0x1
   1417a: 47b68f1b     	addiw	t5, a3, 0x47b
   1417e: 06400393     	li	t2, 0x64
   14182: 05f5e6b7     	lui	a3, 0x5f5e
   14186: 0ff68f9b     	addiw	t6, a3, 0xff
   1418a: 892a         	mv	s2, a0
   1418c: 1502         	slli	a0, a0, 0x20
   1418e: 02653533     	mulhu	a0, a0, t1
   14192: 9135         	srli	a0, a0, 0x2d
   14194: 03c507b3     	mul	a5, a0, t3
   14198: 40f9063b     	subw	a2, s2, a5
   1419c: 03061793     	slli	a5, a2, 0x30
   141a0: 93c9         	srli	a5, a5, 0x32
   141a2: 03e787b3     	mul	a5, a5, t5
   141a6: 0117d493     	srli	s1, a5, 0x11
   141aa: 83c1         	srli	a5, a5, 0x10
   141ac: 7fe7f793     	andi	a5, a5, 0x7fe
   141b0: 027484b3     	mul	s1, s1, t2
   141b4: 9e05         	subw	a2, a2, s1
   141b6: 1646         	slli	a2, a2, 0x31
   141b8: 97f6         	add	a5, a5, t4
   141ba: 0017c483     	lbu	s1, 0x1(a5)
   141be: 9241         	srli	a2, a2, 0x30
   141c0: 00e886b3     	add	a3, a7, a4
   141c4: 0007c783     	lbu	a5, 0x0(a5)
   141c8: 009680a3     	sb	s1, 0x1(a3)
   141cc: 9676         	add	a2, a2, t4
   141ce: 00164483     	lbu	s1, 0x1(a2)
   141d2: 00064603     	lbu	a2, 0x0(a2)
   141d6: 00f68023     	sb	a5, 0x0(a3)
   141da: 00e286b3     	add	a3, t0, a4
   141de: 009680a3     	sb	s1, 0x1(a3)
   141e2: 00c68023     	sb	a2, 0x0(a3)
   141e6: 1771         	addi	a4, a4, -0x4
   141e8: fb2fe1e3     	bltu	t6, s2, 0x1418a <.Lpcrel_hi1041+0x5a>
   141ec: 0729         	addi	a4, a4, 0xa
   141ee: 06300693     	li	a3, 0x63
   141f2: f4a6f9e3     	bgeu	a3, a0, 0x14144 <.Lpcrel_hi1041+0x14>
   141f6: 03051613     	slli	a2, a0, 0x30
   141fa: 9249         	srli	a2, a2, 0x32
   141fc: 6685         	lui	a3, 0x1
   141fe: 47b6869b     	addiw	a3, a3, 0x47b
   14202: 02d60633     	mul	a2, a2, a3
   14206: 8245         	srli	a2, a2, 0x11
   14208: 06400693     	li	a3, 0x64
   1420c: 02d606b3     	mul	a3, a2, a3
   14210: 9d15         	subw	a0, a0, a3
   14212: 1546         	slli	a0, a0, 0x31
   14214: 9141         	srli	a0, a0, 0x30
   14216: 1779         	addi	a4, a4, -0x2
   14218: 9576         	add	a0, a0, t4
   1421a: 00154683     	lbu	a3, 0x1(a0)
   1421e: 00054503     	lbu	a0, 0x0(a0)
   14222: fd640793     	addi	a5, s0, -0x2a
   14226: 97ba         	add	a5, a5, a4
   14228: 00d780a3     	sb	a3, 0x1(a5)
   1422c: 00a78023     	sb	a0, 0x0(a5)
   14230: 8532         	mv	a0, a2
   14232: 4629         	li	a2, 0xa
   14234: f0c56be3     	bltu	a0, a2, 0x1414a <.Lpcrel_hi1041+0x1a>
   14238: 0506         	slli	a0, a0, 0x1
   1423a: ffe70693     	addi	a3, a4, -0x2
   1423e: 9576         	add	a0, a0, t4
   14240: 00154603     	lbu	a2, 0x1(a0)
   14244: 00054503     	lbu	a0, 0x0(a0)
   14248: fd640713     	addi	a4, s0, -0x2a
   1424c: 9736         	add	a4, a4, a3
   1424e: 00c700a3     	sb	a2, 0x1(a4)
   14252: 00a70023     	sb	a0, 0x0(a4)
   14256: fd640713     	addi	a4, s0, -0x2a
   1425a: 9736         	add	a4, a4, a3
   1425c: 47a9         	li	a5, 0xa
   1425e: 8f95         	sub	a5, a5, a3
   14260: 4605         	li	a2, 0x1
   14262: 8542         	mv	a0, a6
   14264: 4681         	li	a3, 0x0
   14266: fffff097     	auipc	ra, 0xfffff
   1426a: ef4080e7     	jalr	-0x10c(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   1426e: 70a2         	ld	ra, 0x28(sp)
   14270: 7402         	ld	s0, 0x20(sp)
   14272: 64e2         	ld	s1, 0x18(sp)
   14274: 6942         	ld	s2, 0x10(sp)
   14276: 6145         	addi	sp, sp, 0x30
   14278: 8082         	ret

000000000001427a <_ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$usize$GT$3fmt17hb7c5519abaf1703fE>:
   1427a: 1141         	addi	sp, sp, -0x10
   1427c: e406         	sd	ra, 0x8(sp)
   1427e: e022         	sd	s0, 0x0(sp)
   14280: 0800         	addi	s0, sp, 0x10
   14282: 6108         	ld	a0, 0x0(a0)
   14284: 862e         	mv	a2, a1
   14286: 4585         	li	a1, 0x1
   14288: 60a2         	ld	ra, 0x8(sp)
   1428a: 6402         	ld	s0, 0x0(sp)
   1428c: 0141         	addi	sp, sp, 0x10
   1428e: 00000317     	auipc	t1, 0x0
   14292: 03830067     	jr	0x38(t1) <_ZN4core3fmt3num3imp21_$LT$impl$u20$u64$GT$4_fmt17he678ad54334687f6E>

0000000000014296 <_ZN4core3fmt3num3imp54_$LT$impl$u20$core..fmt..Display$u20$for$u20$isize$GT$3fmt17h3f90d00ed58c1f73E>:
   14296: 1141         	addi	sp, sp, -0x10
   14298: e406         	sd	ra, 0x8(sp)
   1429a: e022         	sd	s0, 0x0(sp)
   1429c: 0800         	addi	s0, sp, 0x10
   1429e: 6110         	ld	a2, 0x0(a0)
   142a0: 43f65513     	srai	a0, a2, 0x3f
   142a4: 00a646b3     	xor	a3, a2, a0
   142a8: 40a68533     	sub	a0, a3, a0
   142ac: fff64613     	not	a2, a2
   142b0: 927d         	srli	a2, a2, 0x3f
   142b2: 86ae         	mv	a3, a1
   142b4: 85b2         	mv	a1, a2
   142b6: 8636         	mv	a2, a3
   142b8: 60a2         	ld	ra, 0x8(sp)
   142ba: 6402         	ld	s0, 0x0(sp)
   142bc: 0141         	addi	sp, sp, 0x10
   142be: 00000317     	auipc	t1, 0x0
   142c2: 00830067     	jr	0x8(t1) <_ZN4core3fmt3num3imp21_$LT$impl$u20$u64$GT$4_fmt17he678ad54334687f6E>

00000000000142c6 <_ZN4core3fmt3num3imp21_$LT$impl$u20$u64$GT$4_fmt17he678ad54334687f6E>:
   142c6: 7139         	addi	sp, sp, -0x40
   142c8: fc06         	sd	ra, 0x38(sp)
   142ca: f822         	sd	s0, 0x30(sp)
   142cc: f426         	sd	s1, 0x28(sp)
   142ce: f04a         	sd	s2, 0x20(sp)
   142d0: 0080         	addi	s0, sp, 0x40
   142d2: 8832         	mv	a6, a2
   142d4: 00455693     	srli	a3, a0, 0x4
   142d8: 4751         	li	a4, 0x14
   142da: 27100793     	li	a5, 0x271

00000000000142de <.Lpcrel_hi1042>:
   142de: 00002617     	auipc	a2, 0x2
   142e2: c0d60e93     	addi	t4, a2, -0x3f3
   142e6: 02f6f363     	bgeu	a3, a5, 0x1430c <.Lpcrel_hi1042+0x2e>
   142ea: 06300693     	li	a3, 0x63
   142ee: 0aa6e963     	bltu	a3, a0, 0x143a0 <.Lpcrel_hi1043+0x8a>
   142f2: 4629         	li	a2, 0xa
   142f4: 0ec57763     	bgeu	a0, a2, 0x143e2 <.Lpcrel_hi1043+0xcc>
   142f8: fff70693     	addi	a3, a4, -0x1
   142fc: fcc40613     	addi	a2, s0, -0x34
   14300: 9636         	add	a2, a2, a3
   14302: 03056513     	ori	a0, a0, 0x30
   14306: 00a60023     	sb	a0, 0x0(a2)
   1430a: a8dd         	j	0x14400 <.Lpcrel_hi1043+0xea>
   1430c: 4701         	li	a4, 0x0
   1430e: fdc40893     	addi	a7, s0, -0x24
   14312: fde40293     	addi	t0, s0, -0x22

0000000000014316 <.Lpcrel_hi1043>:
   14316: 00001697     	auipc	a3, 0x1
   1431a: 01a6b303     	ld	t1, 0x1a(a3)
   1431e: 6689         	lui	a3, 0x2
   14320: 71068e13     	addi	t3, a3, 0x710
   14324: 6685         	lui	a3, 0x1
   14326: 47b68f1b     	addiw	t5, a3, 0x47b
   1432a: 06400393     	li	t2, 0x64
   1432e: 05f5e6b7     	lui	a3, 0x5f5e
   14332: 0ff68f9b     	addiw	t6, a3, 0xff
   14336: 892a         	mv	s2, a0
   14338: 02653533     	mulhu	a0, a0, t1
   1433c: 812d         	srli	a0, a0, 0xb
   1433e: 03c507b3     	mul	a5, a0, t3
   14342: 40f9063b     	subw	a2, s2, a5
   14346: 03061793     	slli	a5, a2, 0x30
   1434a: 93c9         	srli	a5, a5, 0x32
   1434c: 03e787b3     	mul	a5, a5, t5
   14350: 0117d493     	srli	s1, a5, 0x11
   14354: 83c1         	srli	a5, a5, 0x10
   14356: 7fe7f793     	andi	a5, a5, 0x7fe
   1435a: 027484b3     	mul	s1, s1, t2
   1435e: 9e05         	subw	a2, a2, s1
   14360: 1646         	slli	a2, a2, 0x31
   14362: 97f6         	add	a5, a5, t4
   14364: 0017c483     	lbu	s1, 0x1(a5)
   14368: 9241         	srli	a2, a2, 0x30
   1436a: 00e886b3     	add	a3, a7, a4
   1436e: 0007c783     	lbu	a5, 0x0(a5)
   14372: 009680a3     	sb	s1, 0x1(a3)
   14376: 9676         	add	a2, a2, t4
   14378: 00164483     	lbu	s1, 0x1(a2)
   1437c: 00064603     	lbu	a2, 0x0(a2)
   14380: 00f68023     	sb	a5, 0x0(a3)
   14384: 00e286b3     	add	a3, t0, a4
   14388: 009680a3     	sb	s1, 0x1(a3)
   1438c: 00c68023     	sb	a2, 0x0(a3)
   14390: 1771         	addi	a4, a4, -0x4
   14392: fb2fe2e3     	bltu	t6, s2, 0x14336 <.Lpcrel_hi1043+0x20>
   14396: 0751         	addi	a4, a4, 0x14
   14398: 06300693     	li	a3, 0x63
   1439c: f4a6fbe3     	bgeu	a3, a0, 0x142f2 <.Lpcrel_hi1042+0x14>
   143a0: 03051613     	slli	a2, a0, 0x30
   143a4: 9249         	srli	a2, a2, 0x32
   143a6: 6685         	lui	a3, 0x1
   143a8: 47b6869b     	addiw	a3, a3, 0x47b
   143ac: 02d60633     	mul	a2, a2, a3
   143b0: 8245         	srli	a2, a2, 0x11
   143b2: 06400693     	li	a3, 0x64
   143b6: 02d606b3     	mul	a3, a2, a3
   143ba: 9d15         	subw	a0, a0, a3
   143bc: 1546         	slli	a0, a0, 0x31
   143be: 9141         	srli	a0, a0, 0x30
   143c0: 1779         	addi	a4, a4, -0x2
   143c2: 9576         	add	a0, a0, t4
   143c4: 00154683     	lbu	a3, 0x1(a0)
   143c8: 00054503     	lbu	a0, 0x0(a0)
   143cc: fcc40793     	addi	a5, s0, -0x34
   143d0: 97ba         	add	a5, a5, a4
   143d2: 00d780a3     	sb	a3, 0x1(a5)
   143d6: 00a78023     	sb	a0, 0x0(a5)
   143da: 8532         	mv	a0, a2
   143dc: 4629         	li	a2, 0xa
   143de: f0c56de3     	bltu	a0, a2, 0x142f8 <.Lpcrel_hi1042+0x1a>
   143e2: 0506         	slli	a0, a0, 0x1
   143e4: ffe70693     	addi	a3, a4, -0x2
   143e8: 9576         	add	a0, a0, t4
   143ea: 00154603     	lbu	a2, 0x1(a0)
   143ee: 00054503     	lbu	a0, 0x0(a0)
   143f2: fcc40713     	addi	a4, s0, -0x34
   143f6: 9736         	add	a4, a4, a3
   143f8: 00c700a3     	sb	a2, 0x1(a4)
   143fc: 00a70023     	sb	a0, 0x0(a4)
   14400: fcc40713     	addi	a4, s0, -0x34
   14404: 9736         	add	a4, a4, a3
   14406: 47d1         	li	a5, 0x14
   14408: 8f95         	sub	a5, a5, a3
   1440a: 4605         	li	a2, 0x1
   1440c: 8542         	mv	a0, a6
   1440e: 4681         	li	a3, 0x0
   14410: fffff097     	auipc	ra, 0xfffff
   14414: d4a080e7     	jalr	-0x2b6(ra) <__rust_no_alloc_shim_is_unstable+0xffff4031>
   14418: 70e2         	ld	ra, 0x38(sp)
   1441a: 7442         	ld	s0, 0x30(sp)
   1441c: 74a2         	ld	s1, 0x28(sp)
   1441e: 7902         	ld	s2, 0x20(sp)
   14420: 6121         	addi	sp, sp, 0x40
   14422: 8082         	ret

0000000000014424 <_ZN42_$LT$$RF$T$u20$as$u20$core..fmt..Debug$GT$3fmt17h1a8764df7e6425d0E>:
   14424: 1141         	addi	sp, sp, -0x10
   14426: e406         	sd	ra, 0x8(sp)
   14428: e022         	sd	s0, 0x0(sp)
   1442a: 0800         	addi	s0, sp, 0x10
   1442c: 6510         	ld	a2, 0x8(a0)
   1442e: 6108         	ld	a0, 0x0(a0)
   14430: 6e1c         	ld	a5, 0x18(a2)
   14432: 60a2         	ld	ra, 0x8(sp)
   14434: 6402         	ld	s0, 0x0(sp)
   14436: 0141         	addi	sp, sp, 0x10
   14438: 8782         	jr	a5

000000000001443a <_ZN44_$LT$$RF$T$u20$as$u20$core..fmt..Display$GT$3fmt17h05eab9ec238c4213E>:
   1443a: 1141         	addi	sp, sp, -0x10
   1443c: e406         	sd	ra, 0x8(sp)
   1443e: e022         	sd	s0, 0x0(sp)
   14440: 0800         	addi	s0, sp, 0x10
   14442: 6114         	ld	a3, 0x0(a0)
   14444: 6510         	ld	a2, 0x8(a0)
   14446: 852e         	mv	a0, a1
   14448: 85b6         	mv	a1, a3
   1444a: 60a2         	ld	ra, 0x8(sp)
   1444c: 6402         	ld	s0, 0x0(sp)
   1444e: 0141         	addi	sp, sp, 0x10
   14450: fffff317     	auipc	t1, 0xfffff
   14454: f8e30067     	jr	-0x72(t1) <__rust_no_alloc_shim_is_unstable+0xffff42b5>

0000000000014458 <_ZN4core5slice5index24slice_end_index_len_fail8do_panic7runtime17h77168e5889f9c243E>:
   14458: 7159         	addi	sp, sp, -0x70
   1445a: f486         	sd	ra, 0x68(sp)
   1445c: f0a2         	sd	s0, 0x60(sp)
   1445e: 1880         	addi	s0, sp, 0x70
   14460: f8a43823     	sd	a0, -0x70(s0)
   14464: f8b43c23     	sd	a1, -0x68(s0)
   14468: f9040513     	addi	a0, s0, -0x70
   1446c: fca43823     	sd	a0, -0x30(s0)

0000000000014470 <.Lpcrel_hi1075>:
   14470: 00000517     	auipc	a0, 0x0
   14474: e0a50513     	addi	a0, a0, -0x1f6
   14478: fca43c23     	sd	a0, -0x28(s0)
   1447c: f9840593     	addi	a1, s0, -0x68
   14480: feb43023     	sd	a1, -0x20(s0)
   14484: fea43423     	sd	a0, -0x18(s0)

0000000000014488 <.Lpcrel_hi1076>:
   14488: 00002517     	auipc	a0, 0x2
   1448c: c5050513     	addi	a0, a0, -0x3b0
   14490: faa43023     	sd	a0, -0x60(s0)
   14494: 4509         	li	a0, 0x2
   14496: faa43423     	sd	a0, -0x58(s0)
   1449a: fc043023     	sd	zero, -0x40(s0)
   1449e: fd040593     	addi	a1, s0, -0x30
   144a2: fab43823     	sd	a1, -0x50(s0)
   144a6: faa43c23     	sd	a0, -0x48(s0)
   144aa: fa040513     	addi	a0, s0, -0x60
   144ae: 85b2         	mv	a1, a2
   144b0: ffffe097     	auipc	ra, 0xffffe
   144b4: 488080e7     	jalr	0x488(ra) <__rust_no_alloc_shim_is_unstable+0xffff380f>

00000000000144b8 <_ZN4core5slice5index22slice_index_order_fail8do_panic7runtime17h39c7f1be29aceb7cE>:
   144b8: 7159         	addi	sp, sp, -0x70
   144ba: f486         	sd	ra, 0x68(sp)
   144bc: f0a2         	sd	s0, 0x60(sp)
   144be: 1880         	addi	s0, sp, 0x70
   144c0: f8a43823     	sd	a0, -0x70(s0)
   144c4: f8b43c23     	sd	a1, -0x68(s0)
   144c8: f9040513     	addi	a0, s0, -0x70
   144cc: fca43823     	sd	a0, -0x30(s0)

00000000000144d0 <.Lpcrel_hi1077>:
   144d0: 00000517     	auipc	a0, 0x0
   144d4: daa50513     	addi	a0, a0, -0x256
   144d8: fca43c23     	sd	a0, -0x28(s0)
   144dc: f9840593     	addi	a1, s0, -0x68
   144e0: feb43023     	sd	a1, -0x20(s0)
   144e4: fea43423     	sd	a0, -0x18(s0)

00000000000144e8 <.Lpcrel_hi1078>:
   144e8: 00002517     	auipc	a0, 0x2
   144ec: c3850513     	addi	a0, a0, -0x3c8
   144f0: faa43023     	sd	a0, -0x60(s0)
   144f4: 4509         	li	a0, 0x2
   144f6: faa43423     	sd	a0, -0x58(s0)
   144fa: fc043023     	sd	zero, -0x40(s0)
   144fe: fd040593     	addi	a1, s0, -0x30
   14502: fab43823     	sd	a1, -0x50(s0)
   14506: faa43c23     	sd	a0, -0x48(s0)
   1450a: fa040513     	addi	a0, s0, -0x60
   1450e: 85b2         	mv	a1, a2
   14510: ffffe097     	auipc	ra, 0xffffe
   14514: 428080e7     	jalr	0x428(ra) <__rust_no_alloc_shim_is_unstable+0xffff380f>

0000000000014518 <memcpy>:
   14518: 1141         	addi	sp, sp, -0x10
   1451a: e406         	sd	ra, 0x8(sp)
   1451c: e022         	sd	s0, 0x0(sp)
   1451e: 0800         	addi	s0, sp, 0x10
   14520: 46c1         	li	a3, 0x10
   14522: 06d66263     	bltu	a2, a3, 0x14586 <memcpy+0x6e>
   14526: 40a006bb     	negw	a3, a0
   1452a: 0076f813     	andi	a6, a3, 0x7
   1452e: 01050333     	add	t1, a0, a6
   14532: 00657e63     	bgeu	a0, t1, 0x1454e <memcpy+0x36>
   14536: 88c2         	mv	a7, a6
   14538: 86aa         	mv	a3, a0
   1453a: 872e         	mv	a4, a1
   1453c: 00074783     	lbu	a5, 0x0(a4)
   14540: 00f68023     	sb	a5, 0x0(a3)
   14544: 0685         	addi	a3, a3, 0x1
   14546: 18fd         	addi	a7, a7, -0x1
   14548: 0705         	addi	a4, a4, 0x1
   1454a: fe0899e3     	bnez	a7, 0x1453c <memcpy+0x24>
   1454e: 95c2         	add	a1, a1, a6
   14550: 410603b3     	sub	t2, a2, a6
   14554: ff83f793     	andi	a5, t2, -0x8
   14558: 0075f713     	andi	a4, a1, 0x7
   1455c: 00f306b3     	add	a3, t1, a5
   14560: e721         	bnez	a4, 0x145a8 <memcpy+0x90>
   14562: 00d37a63     	bgeu	t1, a3, 0x14576 <memcpy+0x5e>
   14566: 872e         	mv	a4, a1
   14568: 6310         	ld	a2, 0x0(a4)
   1456a: 00c33023     	sd	a2, 0x0(t1)
   1456e: 0321         	addi	t1, t1, 0x8
   14570: 0721         	addi	a4, a4, 0x8
   14572: fed36be3     	bltu	t1, a3, 0x14568 <memcpy+0x50>
   14576: 95be         	add	a1, a1, a5
   14578: 0073f613     	andi	a2, t2, 0x7
   1457c: 00c68733     	add	a4, a3, a2
   14580: 00e6e863     	bltu	a3, a4, 0x14590 <memcpy+0x78>
   14584: a831         	j	0x145a0 <memcpy+0x88>
   14586: 86aa         	mv	a3, a0
   14588: 00c50733     	add	a4, a0, a2
   1458c: 00e57a63     	bgeu	a0, a4, 0x145a0 <memcpy+0x88>
   14590: 0005c703     	lbu	a4, 0x0(a1)
   14594: 00e68023     	sb	a4, 0x0(a3)
   14598: 0685         	addi	a3, a3, 0x1
   1459a: 167d         	addi	a2, a2, -0x1
   1459c: 0585         	addi	a1, a1, 0x1
   1459e: fa6d         	bnez	a2, 0x14590 <memcpy+0x78>
   145a0: 60a2         	ld	ra, 0x8(sp)
   145a2: 6402         	ld	s0, 0x0(sp)
   145a4: 0141         	addi	sp, sp, 0x10
   145a6: 8082         	ret
   145a8: fcd377e3     	bgeu	t1, a3, 0x14576 <memcpy+0x5e>
   145ac: 00359613     	slli	a2, a1, 0x3
   145b0: 03867813     	andi	a6, a2, 0x38
   145b4: ff85f293     	andi	t0, a1, -0x8
   145b8: 0002b703     	ld	a4, 0x0(t0)
   145bc: 40c0063b     	negw	a2, a2
   145c0: 03867893     	andi	a7, a2, 0x38
   145c4: 02a1         	addi	t0, t0, 0x8
   145c6: 0002b603     	ld	a2, 0x0(t0)
   145ca: 01075e33     	srl	t3, a4, a6
   145ce: 01161733     	sll	a4, a2, a7
   145d2: 01c76733     	or	a4, a4, t3
   145d6: 00e33023     	sd	a4, 0x0(t1)
   145da: 0321         	addi	t1, t1, 0x8
   145dc: 02a1         	addi	t0, t0, 0x8
   145de: 8732         	mv	a4, a2
   145e0: fed363e3     	bltu	t1, a3, 0x145c6 <memcpy+0xae>
   145e4: bf49         	j	0x14576 <memcpy+0x5e>

00000000000145e6 <memcmp>:
   145e6: c605         	beqz	a2, 0x1460e <memcmp+0x28>
   145e8: 1141         	addi	sp, sp, -0x10
   145ea: e406         	sd	ra, 0x8(sp)
   145ec: e022         	sd	s0, 0x0(sp)
   145ee: 0800         	addi	s0, sp, 0x10
   145f0: 00054683     	lbu	a3, 0x0(a0)
   145f4: 0005c703     	lbu	a4, 0x0(a1)
   145f8: 00e69d63     	bne	a3, a4, 0x14612 <memcmp+0x2c>
   145fc: 167d         	addi	a2, a2, -0x1
   145fe: 0585         	addi	a1, a1, 0x1
   14600: 0505         	addi	a0, a0, 0x1
   14602: f67d         	bnez	a2, 0x145f0 <memcmp+0xa>
   14604: 4501         	li	a0, 0x0
   14606: 60a2         	ld	ra, 0x8(sp)
   14608: 6402         	ld	s0, 0x0(sp)
   1460a: 0141         	addi	sp, sp, 0x10
   1460c: 8082         	ret
   1460e: 4501         	li	a0, 0x0
   14610: 8082         	ret
   14612: 40e68533     	sub	a0, a3, a4
   14616: 60a2         	ld	ra, 0x8(sp)
   14618: 6402         	ld	s0, 0x0(sp)
   1461a: 0141         	addi	sp, sp, 0x10
   1461c: 8082         	ret

000000000001461e <memmove>:
   1461e: 1141         	addi	sp, sp, -0x10
   14620: e406         	sd	ra, 0x8(sp)
   14622: e022         	sd	s0, 0x0(sp)
   14624: 0800         	addi	s0, sp, 0x10
   14626: 60a2         	ld	ra, 0x8(sp)
   14628: 6402         	ld	s0, 0x0(sp)
   1462a: 0141         	addi	sp, sp, 0x10
   1462c: 00000317     	auipc	t1, 0x0
   14630: 00830067     	jr	0x8(t1) <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E>

0000000000014634 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E>:
   14634: 1141         	addi	sp, sp, -0x10
   14636: e406         	sd	ra, 0x8(sp)
   14638: e022         	sd	s0, 0x0(sp)
   1463a: 0800         	addi	s0, sp, 0x10
   1463c: 40b506b3     	sub	a3, a0, a1
   14640: 08c6fb63     	bgeu	a3, a2, 0x146d6 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xa2>
   14644: 00c506b3     	add	a3, a0, a2
   14648: 4741         	li	a4, 0x10
   1464a: 00c582b3     	add	t0, a1, a2
   1464e: 06e66463     	bltu	a2, a4, 0x146b6 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x82>
   14652: 0076f813     	andi	a6, a3, 0x7
   14656: ff86fe13     	andi	t3, a3, -0x8
   1465a: 410008b3     	neg	a7, a6
   1465e: 00de7f63     	bgeu	t3, a3, 0x1467c <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x48>
   14662: 00c587b3     	add	a5, a1, a2
   14666: 17fd         	addi	a5, a5, -0x1
   14668: 0007c303     	lbu	t1, 0x0(a5)
   1466c: fff68713     	addi	a4, a3, -0x1
   14670: fe668fa3     	sb	t1, -0x1(a3)
   14674: 17fd         	addi	a5, a5, -0x1
   14676: 86ba         	mv	a3, a4
   14678: feee68e3     	bltu	t3, a4, 0x14668 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x34>
   1467c: 011283b3     	add	t2, t0, a7
   14680: 41060633     	sub	a2, a2, a6
   14684: ff867693     	andi	a3, a2, -0x8
   14688: 0073f793     	andi	a5, t2, 0x7
   1468c: 40d00833     	neg	a6, a3
   14690: 40de06b3     	sub	a3, t3, a3
   14694: e7e9         	bnez	a5, 0x1475e <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x12a>
   14696: 01c6fd63     	bgeu	a3, t3, 0x146b0 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x7c>
   1469a: 95b2         	add	a1, a1, a2
   1469c: 15e1         	addi	a1, a1, -0x8
   1469e: 6198         	ld	a4, 0x0(a1)
   146a0: ff8e0793     	addi	a5, t3, -0x8
   146a4: feee3c23     	sd	a4, -0x8(t3)
   146a8: 15e1         	addi	a1, a1, -0x8
   146aa: 8e3e         	mv	t3, a5
   146ac: fef6e9e3     	bltu	a3, a5, 0x1469e <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x6a>
   146b0: 010382b3     	add	t0, t2, a6
   146b4: 8a1d         	andi	a2, a2, 0x7
   146b6: 40c685b3     	sub	a1, a3, a2
   146ba: 08d5fe63     	bgeu	a1, a3, 0x14756 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x122>
   146be: 12fd         	addi	t0, t0, -0x1
   146c0: 0002c603     	lbu	a2, 0x0(t0)
   146c4: fff68713     	addi	a4, a3, -0x1
   146c8: fec68fa3     	sb	a2, -0x1(a3)
   146cc: 12fd         	addi	t0, t0, -0x1
   146ce: 86ba         	mv	a3, a4
   146d0: fee5e8e3     	bltu	a1, a4, 0x146c0 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x8c>
   146d4: a049         	j	0x14756 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x122>
   146d6: 46c1         	li	a3, 0x10
   146d8: 06d66263     	bltu	a2, a3, 0x1473c <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x108>
   146dc: 40a006bb     	negw	a3, a0
   146e0: 0076f813     	andi	a6, a3, 0x7
   146e4: 01050333     	add	t1, a0, a6
   146e8: 00657e63     	bgeu	a0, t1, 0x14704 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xd0>
   146ec: 88c2         	mv	a7, a6
   146ee: 86aa         	mv	a3, a0
   146f0: 872e         	mv	a4, a1
   146f2: 00074783     	lbu	a5, 0x0(a4)
   146f6: 00f68023     	sb	a5, 0x0(a3)
   146fa: 0685         	addi	a3, a3, 0x1
   146fc: 18fd         	addi	a7, a7, -0x1
   146fe: 0705         	addi	a4, a4, 0x1
   14700: fe0899e3     	bnez	a7, 0x146f2 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xbe>
   14704: 95c2         	add	a1, a1, a6
   14706: 410603b3     	sub	t2, a2, a6
   1470a: ff83f793     	andi	a5, t2, -0x8
   1470e: 0075f713     	andi	a4, a1, 0x7
   14712: 00f306b3     	add	a3, t1, a5
   14716: e749         	bnez	a4, 0x147a0 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x16c>
   14718: 00d37a63     	bgeu	t1, a3, 0x1472c <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xf8>
   1471c: 872e         	mv	a4, a1
   1471e: 6310         	ld	a2, 0x0(a4)
   14720: 00c33023     	sd	a2, 0x0(t1)
   14724: 0321         	addi	t1, t1, 0x8
   14726: 0721         	addi	a4, a4, 0x8
   14728: fed36be3     	bltu	t1, a3, 0x1471e <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xea>
   1472c: 95be         	add	a1, a1, a5
   1472e: 0073f613     	andi	a2, t2, 0x7
   14732: 00c68733     	add	a4, a3, a2
   14736: 00e6e863     	bltu	a3, a4, 0x14746 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x112>
   1473a: a831         	j	0x14756 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x122>
   1473c: 86aa         	mv	a3, a0
   1473e: 00c50733     	add	a4, a0, a2
   14742: 00e57a63     	bgeu	a0, a4, 0x14756 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x122>
   14746: 0005c703     	lbu	a4, 0x0(a1)
   1474a: 00e68023     	sb	a4, 0x0(a3)
   1474e: 0685         	addi	a3, a3, 0x1
   14750: 167d         	addi	a2, a2, -0x1
   14752: 0585         	addi	a1, a1, 0x1
   14754: fa6d         	bnez	a2, 0x14746 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x112>
   14756: 60a2         	ld	ra, 0x8(sp)
   14758: 6402         	ld	s0, 0x0(sp)
   1475a: 0141         	addi	sp, sp, 0x10
   1475c: 8082         	ret
   1475e: f5c6f9e3     	bgeu	a3, t3, 0x146b0 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x7c>
   14762: 00339593     	slli	a1, t2, 0x3
   14766: 0385f893     	andi	a7, a1, 0x38
   1476a: ff83f713     	andi	a4, t2, -0x8
   1476e: 00073303     	ld	t1, 0x0(a4)
   14772: 40b005bb     	negw	a1, a1
   14776: 0385f293     	andi	t0, a1, 0x38
   1477a: ff870593     	addi	a1, a4, -0x8
   1477e: 6198         	ld	a4, 0x0(a1)
   14780: 00531333     	sll	t1, t1, t0
   14784: 011757b3     	srl	a5, a4, a7
   14788: 0067e333     	or	t1, a5, t1
   1478c: ff8e0793     	addi	a5, t3, -0x8
   14790: fe6e3c23     	sd	t1, -0x8(t3)
   14794: 15e1         	addi	a1, a1, -0x8
   14796: 8e3e         	mv	t3, a5
   14798: 833a         	mv	t1, a4
   1479a: fef6e2e3     	bltu	a3, a5, 0x1477e <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x14a>
   1479e: bf09         	j	0x146b0 <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x7c>
   147a0: f8d376e3     	bgeu	t1, a3, 0x1472c <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xf8>
   147a4: 00359613     	slli	a2, a1, 0x3
   147a8: 03867813     	andi	a6, a2, 0x38
   147ac: ff85f293     	andi	t0, a1, -0x8
   147b0: 0002b703     	ld	a4, 0x0(t0)
   147b4: 40c0063b     	negw	a2, a2
   147b8: 03867893     	andi	a7, a2, 0x38
   147bc: 02a1         	addi	t0, t0, 0x8
   147be: 0002b603     	ld	a2, 0x0(t0)
   147c2: 01075e33     	srl	t3, a4, a6
   147c6: 01161733     	sll	a4, a2, a7
   147ca: 01c76733     	or	a4, a4, t3
   147ce: 00e33023     	sd	a4, 0x0(t1)
   147d2: 0321         	addi	t1, t1, 0x8
   147d4: 02a1         	addi	t0, t0, 0x8
   147d6: 8732         	mv	a4, a2
   147d8: fed363e3     	bltu	t1, a3, 0x147be <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0x18a>
   147dc: bf81         	j	0x1472c <_ZN17compiler_builtins3mem7memmove17he093806ef1bed350E+0xf8>
